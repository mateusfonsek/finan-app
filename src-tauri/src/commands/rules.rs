use rusqlite::params;
use tauri::State;

use crate::db::Db;
use crate::domain::rule::{CalendarEvent, NewRule, Rule, UpdateRule};
use crate::error::{AppError, AppResult};

fn validate_due_day(d: Option<i32>) -> AppResult<()> {
    if let Some(day) = d {
        if !(1..=31).contains(&day) {
            return Err(AppError::Invalid(format!(
                "due_day deve estar entre 1 e 31 (recebido: {day})"
            )));
        }
    }
    Ok(())
}

/// Normaliza a lista de trechos vinda da UI: apara espaços, descarta vazios e
/// remove duplicatas (comparando sem caixa, que é como o casamento funciona).
/// A ordem que o usuário digitou é preservada.
fn clean_patterns(raw: &[String]) -> AppResult<Vec<String>> {
    let mut out: Vec<String> = Vec::with_capacity(raw.len());
    for p in raw {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        if out.iter().any(|kept| kept.eq_ignore_ascii_case(p)) {
            continue;
        }
        out.push(p.to_string());
    }
    if out.is_empty() {
        return Err(AppError::Invalid("rule must have at least one pattern".into()));
    }
    Ok(out)
}

/// Regrava os trechos de uma regra. Substitui em bloco em vez de fazer diff:
/// a lista é curta e a ordem importa, então recriar é mais simples de acertar.
fn replace_patterns(tx: &rusqlite::Transaction, rule_id: i64, patterns: &[String]) -> AppResult<()> {
    tx.execute("DELETE FROM rule_patterns WHERE rule_id = ?1", params![rule_id])?;
    let mut stmt = tx.prepare("INSERT INTO rule_patterns (rule_id, pattern) VALUES (?1, ?2)")?;
    for p in patterns {
        stmt.execute(params![rule_id, p])?;
    }
    Ok(())
}

/// Trechos de uma regra, na ordem em que foram gravados.
fn patterns_of(conn: &rusqlite::Connection, rule_id: i64) -> rusqlite::Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT pattern FROM rule_patterns WHERE rule_id = ?1 ORDER BY id")?;
    let rows = stmt.query_map(params![rule_id], |row| row.get::<_, String>(0))?;
    rows.collect()
}

#[tauri::command]
#[specta::specta]
pub fn list_rules(db: State<'_, Db>) -> AppResult<Vec<Rule>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let mut stmt = conn.prepare(
        "SELECT id, category_id, priority, due_day, display_name, created_at
         FROM rules
         ORDER BY priority DESC, created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        Ok(Rule {
            id,
            patterns: patterns_of(&conn, id)?,
            category_id: row.get(1)?,
            priority: row.get(2)?,
            due_day: row.get(3)?,
            display_name: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn create_rule(db: State<'_, Db>, input: NewRule) -> AppResult<Rule> {
    let patterns = clean_patterns(&input.patterns)?;
    validate_due_day(input.due_day)?;
    let mut conn = db.conn.lock().expect("db mutex poisoned");

    let id = {
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO rules (category_id, priority, due_day, display_name)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                input.category_id,
                input.priority,
                input.due_day,
                input.display_name.as_deref().map(str::trim),
            ],
        )?;
        let id = tx.last_insert_rowid();
        replace_patterns(&tx, id, &patterns)?;
        tx.commit()?;
        id
    };

    apply_rules_internal(&mut conn, None)?;
    fetch_rule(&conn, id)
}

#[tauri::command]
#[specta::specta]
pub fn update_rule(db: State<'_, Db>, rule_id: i64, input: UpdateRule) -> AppResult<Rule> {
    let patterns = clean_patterns(&input.patterns)?;
    validate_due_day(input.due_day)?;
    let mut conn = db.conn.lock().expect("db mutex poisoned");

    {
        let tx = conn.transaction()?;
        let changed = tx.execute(
            "UPDATE rules
             SET category_id = ?1, priority = ?2, due_day = ?3, display_name = ?4
             WHERE id = ?5",
            params![
                input.category_id,
                input.priority,
                input.due_day,
                input.display_name.as_deref().map(str::trim),
                rule_id
            ],
        )?;
        if changed == 0 {
            return Err(AppError::Invalid(format!("rule {rule_id} not found")));
        }
        replace_patterns(&tx, rule_id, &patterns)?;
        tx.commit()?;
    }

    apply_rules_internal(&mut conn, None)?;
    fetch_rule(&conn, rule_id)
}

#[tauri::command]
#[specta::specta]
pub fn delete_rule(db: State<'_, Db>, rule_id: i64) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let changed = conn.execute("DELETE FROM rules WHERE id = ?1", params![rule_id])?;
    if changed == 0 {
        return Err(AppError::Invalid(format!("rule {rule_id} not found")));
    }
    Ok(())
}

/// Deletes a rule AND clears category_id from any transaction that was likely
/// categorized BY this rule (description matches ANY of its patterns +
/// category_id is this rule's category). Then re-applies remaining rules to
/// pick alternatives.
///
/// Used by the import screen when the user wants to undo an auto-created rule.
/// Returns the count of transactions whose category was cleared.
#[tauri::command]
#[specta::specta]
pub fn delete_rule_with_cleanup(db: State<'_, Db>, rule_id: i64) -> AppResult<u32> {
    let mut conn = db.conn.lock().expect("db mutex poisoned");
    let category_id: i64 = conn
        .query_row(
            "SELECT category_id FROM rules WHERE id = ?1",
            params![rule_id],
            |r| r.get(0),
        )
        .map_err(|_| AppError::Invalid(format!("rule {rule_id} not found")))?;
    let patterns = patterns_of(&conn, rule_id)?;

    let tx = conn.transaction()?;
    // Limpa antes de apagar: o ON DELETE CASCADE levaria os patterns junto e a
    // consulta abaixo não teria mais como saber o que essa regra categorizou.
    let mut cleared = 0usize;
    for p in &patterns {
        cleared += tx.execute(
            "UPDATE transactions
             SET category_id = NULL
             WHERE category_id = ?1
               AND LOWER(description) LIKE '%' || LOWER(?2) || '%'",
            params![category_id, p],
        )?;
    }
    tx.execute("DELETE FROM rules WHERE id = ?1", params![rule_id])?;
    tx.commit()?;

    // Re-apply remaining rules — a previously-shadowed rule may now match.
    apply_rules_internal(&mut conn, None)?;
    Ok(cleared as u32)
}

/// Run all rules on transactions with `category_id IS NULL` (manual categorization
/// is never overwritten). When multiple rules match the same transaction, the one
/// with the highest priority wins; ties broken by most recent created_at.
/// Returns the count of transactions newly categorized.
#[tauri::command]
#[specta::specta]
pub fn apply_rules_to_uncategorized(db: State<'_, Db>, account_id: Option<i64>) -> AppResult<u32> {
    let mut conn = db.conn.lock().expect("db mutex poisoned");
    apply_rules_internal(&mut conn, account_id)
}

/// Shared engine used both by the public command and by insert_transactions.
pub fn apply_rules_internal(
    conn: &mut rusqlite::Connection,
    account_id: Option<i64>,
) -> AppResult<u32> {
    let scope_filter = match account_id {
        Some(_) => "AND account_id = ?1",
        None => "",
    };
    // Uma regra casa quando QUALQUER um dos seus trechos aparece na descrição.
    let sql = format!(
        "UPDATE transactions
         SET category_id = (
             SELECT r.category_id FROM rules r
             WHERE EXISTS (
                 SELECT 1 FROM rule_patterns p
                 WHERE p.rule_id = r.id
                   AND LOWER(transactions.description) LIKE '%' || LOWER(p.pattern) || '%'
             )
             ORDER BY r.priority DESC, r.created_at DESC
             LIMIT 1
         )
         WHERE category_id IS NULL
           AND EXISTS (
               SELECT 1 FROM rules r
               JOIN rule_patterns p ON p.rule_id = r.id
               WHERE LOWER(transactions.description) LIKE '%' || LOWER(p.pattern) || '%'
           )
           {scope_filter}",
    );
    let changed = if let Some(id) = account_id {
        conn.execute(&sql, params![id])?
    } else {
        conn.execute(&sql, [])?
    };
    Ok(changed as u32)
}

fn fetch_rule(conn: &rusqlite::Connection, id: i64) -> AppResult<Rule> {
    let patterns = patterns_of(conn, id)?;
    conn.query_row(
        "SELECT id, category_id, priority, due_day, display_name, created_at
         FROM rules WHERE id = ?1",
        params![id],
        |row| {
            Ok(Rule {
                id: row.get(0)?,
                patterns,
                category_id: row.get(1)?,
                priority: row.get(2)?,
                due_day: row.get(3)?,
                display_name: row.get(4)?,
                created_at: row.get(5)?,
            })
        },
    )
    .map_err(AppError::from)
}

/// Cruza regras × transações do mês pra montar eventos do calendário.
///
/// Para cada regra:
/// - Se `due_day` set: gera evento com vencimento (mesmo sem casar transação)
/// - Se houver transação no mês cujo description casa o pattern: enriquece
///   o evento com paid_day + paid_amount + paid_transaction_id
/// - Se due_day=NULL e sem match: regra NÃO aparece (semântica do usuário:
///   "só aparece quando paga")
///
/// Quando múltiplas transações casam a mesma regra no mesmo mês, pega a
/// primeira (data ascendente).
#[tauri::command]
#[specta::specta]
pub fn calendar_events(db: State<'_, Db>, month: String) -> AppResult<Vec<CalendarEvent>> {
    if month.len() != 7 || !month.contains('-') {
        return Err(AppError::Invalid(format!(
            "month deve ser 'YYYY-MM' (recebido: '{month}')"
        )));
    }

    let conn = db.conn.lock().expect("db mutex poisoned");
    let date_prefix = format!("{month}-%");

    // Step 1: load all rules with category info + their patterns.
    type RuleRow = (i64, Vec<String>, Option<i32>, String, Option<String>);
    let rule_rows: Vec<RuleRow> = {
        let mut stmt = conn.prepare(
            "SELECT r.id, r.due_day, c.name, c.color_token
             FROM rules r
             JOIN categories c ON c.id = r.category_id
             ORDER BY r.created_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                Ok((
                    id,
                    patterns_of(&conn, id)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows
    };

    // Step 2: load transactions of the month (id, date, amount, description).
    type TxRow = (i64, String, String, String);
    let tx_rows: Vec<TxRow> = {
        let mut stmt = conn.prepare(
            "SELECT id, date, amount, description
             FROM transactions
             WHERE date LIKE ?1
             ORDER BY date ASC, id ASC",
        )?;
        let rows = stmt
            .query_map(params![date_prefix], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows
    };

    // Step 3: for each rule, find first matching tx in the month.
    let mut events: Vec<CalendarEvent> = Vec::new();
    for (rule_id, patterns, due_day, cat_name, cat_color) in rule_rows {
        let patterns_lc: Vec<String> = patterns.iter().map(|p| p.to_lowercase()).collect();
        // Percorre as transações por data: a primeira do mês que casar QUALQUER
        // trecho é a que paga o evento. Rodar por transação (e não por trecho)
        // mantém "a mais antiga vence" mesmo com vários trechos.
        let matched = tx_rows.iter().find_map(|(tx_id, date, amount, desc)| {
            let desc_lc = desc.to_lowercase();
            let hit = patterns_lc.iter().position(|p| desc_lc.contains(p))?;
            Some((tx_id, date, amount, hit))
        });

        // Rótulo do evento: o trecho que casou, ou o primeiro quando o evento
        // existe só pelo vencimento.
        let label = match matched {
            Some((_, _, _, hit)) => patterns[hit].clone(),
            None => patterns.first().cloned().unwrap_or_default(),
        };

        let (paid_day, paid_amount, paid_tx_id) = match matched {
            Some((tx_id, date, amount, _)) => {
                let day: Option<i32> = date.get(8..10).and_then(|s| s.parse().ok());
                (day, Some(amount.clone()), Some(*tx_id))
            }
            None => (None, None, None),
        };

        // Mostra a regra se tem due_day OU se casou alguma transação.
        if due_day.is_some() || paid_tx_id.is_some() {
            events.push(CalendarEvent {
                rule_id,
                pattern: label,
                category_name: cat_name,
                category_color_token: cat_color,
                due_day,
                paid_day,
                paid_amount,
                paid_transaction_id: paid_tx_id,
            });
        }
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::apply_rules_internal;
    use crate::db::migrations;
    use rusqlite::{params, Connection};

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    fn category_id(conn: &Connection, name: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM categories WHERE name = ?1",
            params![name],
            |r| r.get(0),
        )
        .unwrap()
    }

    fn insert_account(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO accounts (name, bank, ofx_acctid) VALUES ('test', NULL, 'A1')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_tx(
        conn: &Connection,
        account_id: i64,
        description: &str,
        category_id: Option<i64>,
    ) -> i64 {
        conn.execute(
            "INSERT INTO transactions (account_id, date, amount, description, category_id, ofx_fitid)
             VALUES (?1, '2026-04-12', '10.00', ?2, ?3, NULL)",
            params![account_id, description, category_id],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_rule(conn: &Connection, pattern: &str, cat: i64, priority: i32) -> i64 {
        insert_rule_multi(conn, &[pattern], cat, priority)
    }

    fn insert_rule_multi(conn: &Connection, patterns: &[&str], cat: i64, priority: i32) -> i64 {
        conn.execute(
            "INSERT INTO rules (category_id, priority, due_day) VALUES (?1, ?2, NULL)",
            params![cat, priority],
        )
        .unwrap();
        let id = conn.last_insert_rowid();
        for p in patterns {
            conn.execute(
                "INSERT INTO rule_patterns (rule_id, pattern) VALUES (?1, ?2)",
                params![id, p],
            )
            .unwrap();
        }
        id
    }

    #[test]
    fn applies_rule_case_insensitive() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let transporte = category_id(&conn, "Transporte");
        insert_rule(&conn, "testmerchant", transporte, 0);
        let tx_id = insert_tx(&conn, acc, "TESTMERCHANT * TRIP 12345", None);

        let n = apply_rules_internal(&mut conn, None).unwrap();
        assert_eq!(n, 1);

        let cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE id = ?1",
                params![tx_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cat, Some(transporte));
    }

    #[test]
    fn higher_priority_rule_wins() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let transporte = category_id(&conn, "Transporte");
        let outros = category_id(&conn, "Outros");

        insert_rule(&conn, "testmerchant", outros, 0);
        insert_rule(&conn, "testmerchant trip", transporte, 10);

        let tx_id = insert_tx(&conn, acc, "uber trip via app", None);
        apply_rules_internal(&mut conn, None).unwrap();

        let cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE id = ?1",
                params![tx_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cat, Some(transporte), "priority 10 wins over priority 0");
    }

    #[test]
    fn manual_categorization_is_never_overwritten() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let mercado = category_id(&conn, "Mercado");
        let transporte = category_id(&conn, "Transporte");

        insert_rule(&conn, "testmerchant", transporte, 0);
        let tx_id = insert_tx(&conn, acc, "TESTMERCHANT trip", Some(mercado));

        let n = apply_rules_internal(&mut conn, None).unwrap();
        assert_eq!(n, 0, "manual category preserved");

        let cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE id = ?1",
                params![tx_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cat, Some(mercado));
    }

    #[test]
    fn scope_to_account_id() {
        let mut conn = fresh_conn();
        conn.execute(
            "INSERT INTO accounts (name, bank, ofx_acctid) VALUES ('a1', NULL, 'A1'),
                                                                  ('a2', NULL, 'A2')",
            [],
        )
        .unwrap();
        let a1: i64 = conn
            .query_row("SELECT id FROM accounts WHERE ofx_acctid='A1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        let a2: i64 = conn
            .query_row("SELECT id FROM accounts WHERE ofx_acctid='A2'", [], |r| {
                r.get(0)
            })
            .unwrap();
        let transporte = category_id(&conn, "Transporte");
        insert_rule(&conn, "testmerchant", transporte, 0);
        insert_tx(&conn, a1, "testmerchant a1", None);
        insert_tx(&conn, a2, "testmerchant a2", None);

        let n = apply_rules_internal(&mut conn, Some(a1)).unwrap();
        assert_eq!(n, 1, "scope must limit to account a1");

        let a1_cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE account_id = ?1",
                params![a1],
                |r| r.get(0),
            )
            .unwrap();
        let a2_cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE account_id = ?1",
                params![a2],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(a1_cat, Some(transporte));
        assert_eq!(a2_cat, None);
    }

    /// Simulates delete_rule_with_cleanup logic (the tauri command needs State,
    /// so we replicate the SQL here against a raw connection).
    fn cleanup_after_delete(conn: &mut Connection, rule_id: i64) -> u32 {
        let category_id: i64 = conn
            .query_row(
                "SELECT category_id FROM rules WHERE id = ?1",
                params![rule_id],
                |r| r.get(0),
            )
            .unwrap();
        let patterns = super::patterns_of(conn, rule_id).unwrap();
        let tx = conn.transaction().unwrap();
        let mut cleared = 0usize;
        for p in &patterns {
            cleared += tx
                .execute(
                    "UPDATE transactions SET category_id = NULL
                     WHERE category_id = ?1 AND LOWER(description) LIKE '%' || LOWER(?2) || '%'",
                    params![category_id, p],
                )
                .unwrap();
        }
        tx.execute("DELETE FROM rules WHERE id = ?1", params![rule_id])
            .unwrap();
        tx.commit().unwrap();
        apply_rules_internal(conn, None).unwrap();
        cleared as u32
    }

    #[test]
    fn delete_with_cleanup_clears_matching_txs() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let transporte = category_id(&conn, "Transporte");
        let rule = insert_rule(&conn, "testmerchant", transporte, 0);
        let tx_id = insert_tx(&conn, acc, "TESTMERCHANT trip", None);
        apply_rules_internal(&mut conn, None).unwrap();
        let cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE id = ?1",
                params![tx_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cat, Some(transporte), "precondition: classified by rule");

        let cleared = cleanup_after_delete(&mut conn, rule);
        assert_eq!(cleared, 1);

        let cat_after: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE id = ?1",
                params![tx_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cat_after, None, "category cleared after rule deletion");
    }

    #[test]
    fn delete_with_cleanup_preserves_manual_overrides() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let transporte = category_id(&conn, "Transporte");
        let mercado = category_id(&conn, "Mercado");
        let rule = insert_rule(&conn, "testmerchant", transporte, 0);
        // Manually categorized as Mercado (different from rule's category) — shouldn't be cleared.
        let tx_id = insert_tx(&conn, acc, "TESTMERCHANT trip", Some(mercado));

        cleanup_after_delete(&mut conn, rule);

        let cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE id = ?1",
                params![tx_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cat, Some(mercado), "manual category Mercado preserved");
    }

    #[test]
    fn pattern_with_no_match_does_nothing() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let transporte = category_id(&conn, "Transporte");
        insert_rule(&conn, "testmerchant", transporte, 0);
        insert_tx(&conn, acc, "padaria do bairro", None);

        let n = apply_rules_internal(&mut conn, None).unwrap();
        assert_eq!(n, 0);
    }

    /// O caso que motivou os múltiplos trechos: o mesmo débito aparece no
    /// extrato ora como débito da conta, ora como pagamento de boleto.
    #[test]
    fn any_pattern_of_a_rule_matches() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let moradia = category_id(&conn, "Casa");
        insert_rule_multi(
            &conn,
            &["CAIXA ECONOMICA FEDERAL (0104)", "GCI CAIXA - HABITACAO"],
            moradia,
            0,
        );
        let a = insert_tx(&conn, acc, "CAIXA ECONOMICA FEDERAL (0104) Agencia: 37", None);
        let b = insert_tx(
            &conn,
            acc,
            "Pagamento de boleto efetuado - GCI CAIXA - HABITACAO",
            None,
        );
        let c = insert_tx(&conn, acc, "padaria do bairro", None);

        let n = apply_rules_internal(&mut conn, None).unwrap();
        assert_eq!(n, 2, "os dois formatos casam a mesma regra");

        for id in [a, b] {
            let cat: Option<i64> = conn
                .query_row(
                    "SELECT category_id FROM transactions WHERE id = ?1",
                    params![id],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(cat, Some(moradia));
        }
        let untouched: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE id = ?1",
                params![c],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(untouched, None);
    }

    /// Prioridade continua sendo da REGRA, não do trecho: um trecho genérico
    /// numa regra de prioridade alta ganha de um específico numa regra baixa.
    #[test]
    fn priority_is_per_rule_not_per_pattern() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let transporte = category_id(&conn, "Transporte");
        let outros = category_id(&conn, "Outros");

        insert_rule_multi(&conn, &["nunca-casa", "testmerchant"], transporte, 10);
        insert_rule(&conn, "testmerchant trip", outros, 0);

        let tx_id = insert_tx(&conn, acc, "TESTMERCHANT TRIP 99", None);
        apply_rules_internal(&mut conn, None).unwrap();

        let cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE id = ?1",
                params![tx_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cat, Some(transporte));
    }

    /// Apagar a regra tem que limpar o que QUALQUER um dos trechos categorizou.
    #[test]
    fn delete_with_cleanup_clears_every_pattern() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let moradia = category_id(&conn, "Casa");
        let rule = insert_rule_multi(&conn, &["alpha-pattern", "beta-pattern"], moradia, 0);
        let a = insert_tx(&conn, acc, "cobranca ALPHA-PATTERN 1", None);
        let b = insert_tx(&conn, acc, "cobranca BETA-PATTERN 2", None);
        apply_rules_internal(&mut conn, None).unwrap();

        let cleared = cleanup_after_delete(&mut conn, rule);
        assert_eq!(cleared, 2);

        for id in [a, b] {
            let cat: Option<i64> = conn
                .query_row(
                    "SELECT category_id FROM transactions WHERE id = ?1",
                    params![id],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(cat, None);
        }
    }

    #[test]
    fn clean_patterns_trims_dedupes_and_rejects_empty() {
        use super::clean_patterns;

        let out = clean_patterns(&[
            "  uber  ".into(),
            "UBER".into(),
            "".into(),
            "   ".into(),
            "99pop".into(),
        ])
        .unwrap();
        assert_eq!(out, vec!["uber".to_string(), "99pop".to_string()]);

        assert!(clean_patterns(&[]).is_err());
        assert!(clean_patterns(&["   ".into()]).is_err());
    }

}
