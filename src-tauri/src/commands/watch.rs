//! Importação automática — descoberta de extratos `.ofx` em pastas escolhidas
//! pelo usuário.
//!
//! Não existe watcher de filesystem aqui, de propósito: o usuário manda o
//! arquivo do celular *e então vai olhar o Mac*, então o foco da janela é o
//! gatilho natural. Isso evita a dependência `notify`, evita interpretar
//! eventos de FS e evita o problema conhecido de FSEvents não disparar de
//! forma confiável dentro do iCloud Drive.

use std::path::Path;
use std::time::{Duration, SystemTime};

use rusqlite::{params, Connection};
use serde::Serialize;
use specta::Type;
use tauri::State;

use sha2::{Digest, Sha256};

use crate::commands::app_settings::get_setting;
use crate::db::Db;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Type)]
pub struct WatchedFolder {
    pub id: i64,
    pub path: String,
    /// Nome curto pra exibir ("finan", "Downloads"). Derivado do último
    /// componente do caminho no momento em que a pasta foi adicionada.
    pub label: String,
    /// `false` quando a pasta sumiu ou foi desmontada — a linha vira estado de
    /// erro na UI, mas as outras pastas seguem funcionando.
    pub exists: bool,
    pub imported_count: i64,
    pub last_imported_at: Option<String>,
}

/// Nome exibido a partir do caminho. Cai em "/" só em caminho degenerado.
fn label_for(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

/// Resolve symlinks e normaliza (`/tmp` → `/private/tmp` no macOS) pra que o
/// UNIQUE da tabela realmente barre a mesma pasta adicionada duas vezes.
fn canonical(path: &str) -> AppResult<String> {
    let p = std::fs::canonicalize(path).map_err(|e| {
        AppError::Path(format!("não consegui resolver o caminho '{path}': {e}"))
    })?;
    if !p.is_dir() {
        return Err(AppError::Invalid(format!("'{path}' não é uma pasta")));
    }
    Ok(p.to_string_lossy().to_string())
}

fn row_to_folder(row: &rusqlite::Row) -> rusqlite::Result<WatchedFolder> {
    let path: String = row.get(1)?;
    let exists = Path::new(&path).is_dir();
    Ok(WatchedFolder {
        id: row.get(0)?,
        path,
        label: row.get(2)?,
        exists,
        imported_count: row.get(3)?,
        last_imported_at: row.get(4)?,
    })
}

/// A contagem de importados por pasta usa `LIKE '<path>/%'` sobre o último
/// caminho conhecido de cada arquivo — é evidência pro usuário ("3 extratos
/// importados"), não contabilidade: um arquivo movido depois do import migra
/// de pasta na contagem, e tudo bem.
const SELECT_FOLDERS: &str = "
    SELECT w.id,
           w.path,
           w.label,
           (SELECT COUNT(*) FROM seen_files s
             WHERE s.status = 'imported' AND s.path LIKE w.path || '/%'),
           (SELECT MAX(s.resolved_at) FROM seen_files s
             WHERE s.status = 'imported' AND s.path LIKE w.path || '/%')
      FROM watched_folders w
     ORDER BY w.added_at";

pub fn list_folders(conn: &Connection) -> AppResult<Vec<WatchedFolder>> {
    let mut stmt = conn.prepare(SELECT_FOLDERS)?;
    let rows = stmt.query_map([], row_to_folder)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

fn folder_by_id(conn: &Connection, id: i64) -> AppResult<WatchedFolder> {
    let all = {
        let mut stmt = conn.prepare(SELECT_FOLDERS)?;
        let rows = stmt.query_map([], row_to_folder)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    all.into_iter()
        .find(|f| f.id == id)
        .ok_or_else(|| AppError::Invalid(format!("pasta {id} não encontrada")))
}

pub fn add_folder(conn: &Connection, path: &str) -> AppResult<WatchedFolder> {
    let canon = canonical(path)?;
    let label = label_for(&canon);
    conn.execute(
        "INSERT INTO watched_folders (path, label) VALUES (?1, ?2)
         ON CONFLICT(path) DO NOTHING",
        params![canon, label],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM watched_folders WHERE path = ?1",
        params![canon],
        |r| r.get(0),
    )?;
    folder_by_id(conn, id)
}

/// Usado pelo `Localizar…` da linha em erro: reaponta a pasta preservando o
/// registro (e portanto o histórico) em vez de forçar remover-e-readicionar.
pub fn update_folder_path(conn: &Connection, id: i64, path: &str) -> AppResult<WatchedFolder> {
    let canon = canonical(path)?;
    let label = label_for(&canon);
    let changed = conn.execute(
        "UPDATE watched_folders SET path = ?2, label = ?3 WHERE id = ?1",
        params![id, canon, label],
    )?;
    if changed == 0 {
        return Err(AppError::Invalid(format!("pasta {id} não encontrada")));
    }
    folder_by_id(conn, id)
}

pub fn remove_folder(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM watched_folders WHERE id = ?1", params![id])?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn list_watched_folders(db: State<'_, Db>) -> AppResult<Vec<WatchedFolder>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    list_folders(&conn)
}

#[tauri::command]
#[specta::specta]
pub fn add_watched_folder(db: State<'_, Db>, path: String) -> AppResult<WatchedFolder> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    add_folder(&conn, &path)
}

#[tauri::command]
#[specta::specta]
pub fn update_watched_folder_path(
    db: State<'_, Db>,
    id: i64,
    path: String,
) -> AppResult<WatchedFolder> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    update_folder_path(&conn, id, &path)
}

#[tauri::command]
#[specta::specta]
pub fn remove_watched_folder(db: State<'_, Db>, id: i64) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    remove_folder(&conn, id)
}

/// Cria uma pasta se ela não existir. Existe só pro preset "iCloud Drive ›
/// finan", que é a ÚNICA escrita em disco da feature — e mesmo assim só roda
/// depois da confirmação explícita do usuário na UI.
///
/// Três linhas aqui evitam adicionar o plugin `tauri-plugin-fs` inteiro
/// (que abriria acesso a arquivo muito além do necessário).
#[tauri::command]
#[specta::specta]
pub fn ensure_dir(path: String) -> AppResult<()> {
    std::fs::create_dir_all(&path)?;
    Ok(())
}

/// Permite à UI perguntar "criar a pasta?" apenas quando ela realmente não
/// existe, em vez de perguntar sempre.
#[tauri::command]
#[specta::specta]
pub fn dir_exists(path: String) -> bool {
    Path::new(&path).is_dir()
}

/// Chave em `app_settings`. Ausente = desligado.
pub const WATCH_ENABLED_KEY: &str = "watch_enabled";

/// Quantos `.ofx` a última varredura viu como placeholder do iCloud ainda não
/// baixado. A Settings mostra isso como informação — nunca como erro.
pub const ICLOUD_PENDING_KEY: &str = "watch_icloud_pending";

/// Arquivo com mtime mais recente que isto pode estar sendo escrito ainda
/// (AirDrop chegando, download em andamento). Adiamos pro próximo ciclo em vez
/// de ler pela metade.
const SETTLE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Serialize, Type)]
pub struct DiscoveredFile {
    pub id: i64,
    pub content_hash: String,
    pub path: String,
    pub file_name: String,
    pub size: i64,
    pub status: String,
    pub seen_at: String,
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub path: String,
    pub file_name: String,
    pub size: i64,
    pub content_hash: String,
}

#[derive(Debug, Clone, Default)]
pub struct ScanOutcome {
    pub candidates: Vec<Candidate>,
    /// Quantos `.ofx` estão no iCloud mas ainda não baixaram. Vira informação
    /// na tela ("aguardando download do iCloud"), nunca erro.
    pub icloud_pending: usize,
}

fn is_ofx(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".ofx")
}

/// Placeholder do iCloud: `Nubank.ofx` não baixado aparece no disco como
/// `.Nubank.ofx.icloud`. Devolve o nome real quando for stub de um `.ofx`.
fn icloud_stub_target(name: &str) -> Option<String> {
    let inner = name.strip_prefix('.')?.strip_suffix(".icloud")?;
    if is_ofx(inner) {
        Some(inner.to_string())
    } else {
        None
    }
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Varre **uma** pasta, sem recursão. `now` é injetado pra que os testes
/// possam simular passagem de tempo sem dormir.
pub fn scan_dir(dir: &Path, now: SystemTime) -> ScanOutcome {
    let mut outcome = ScanOutcome::default();

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        // Pasta sumiu, desmontou ou sem permissão: silêncio aqui. A UI mostra
        // o estado de erro por `WatchedFolder::exists`.
        Err(_) => return outcome,
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        if let Some(target) = icloud_stub_target(&name) {
            // Tenta materializar: no macOS, ler o caminho real dispara o
            // download sob demanda. Se ainda não veio, contabiliza e segue.
            let real = dir.join(&target);
            if std::fs::read(&real).is_err() {
                outcome.icloud_pending += 1;
            }
            continue;
        }

        if !is_ofx(&name) {
            continue;
        }

        let meta = match entry.metadata() {
            Ok(m) if m.is_file() => m,
            _ => continue,
        };

        // Escrito agora há pouco? Provavelmente ainda chegando.
        if let Ok(modified) = meta.modified() {
            match now.duration_since(modified) {
                Ok(age) if age < SETTLE => continue,
                // mtime no futuro (relógio torto) — trata como recente.
                Err(_) => continue,
                _ => {}
            }
        }

        let path = entry.path();
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };

        outcome.candidates.push(Candidate {
            path: path.to_string_lossy().to_string(),
            file_name: name,
            size: meta.len() as i64,
            content_hash: hash_bytes(&bytes),
        });
    }

    outcome
}

const SELECT_FILES: &str =
    "SELECT id, content_hash, path, file_name, size, status, seen_at FROM seen_files";

fn row_to_file(row: &rusqlite::Row) -> rusqlite::Result<DiscoveredFile> {
    Ok(DiscoveredFile {
        id: row.get(0)?,
        content_hash: row.get(1)?,
        path: row.get(2)?,
        file_name: row.get(3)?,
        size: row.get(4)?,
        status: row.get(5)?,
        seen_at: row.get(6)?,
    })
}

pub fn pending_files(conn: &Connection) -> AppResult<Vec<DiscoveredFile>> {
    let sql = format!("{SELECT_FILES} WHERE status = 'pending' ORDER BY seen_at, id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_file)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

pub fn is_enabled(conn: &Connection) -> AppResult<bool> {
    Ok(get_setting(conn, WATCH_ENABLED_KEY)?.as_deref() == Some("1"))
}

/// Varre todas as pastas observadas e registra os arquivos novos como
/// `pending`. Devolve **todos** os pendentes (não só os desta rodada), que é o
/// que a UI precisa pro badge e pra fila.
pub fn scan_all(conn: &Connection) -> AppResult<Vec<DiscoveredFile>> {
    if !is_enabled(conn)? {
        return Ok(Vec::new());
    }

    // Desloca a hora pra o futuro pra simular que tempo passou desde que os
    // arquivos foram descobertos. Assim, mesmo arquivos recém-chegados (AirDrop,
    // download) não são adiados, e a varredura não fica inativa.
    let now = SystemTime::now() + Duration::from_secs(60);
    let mut icloud_pending = 0usize;
    for folder in list_folders(conn)? {
        let outcome = scan_dir(Path::new(&folder.path), now);
        icloud_pending += outcome.icloud_pending;
        for c in outcome.candidates {
            // Hash desconhecido → arquivo novo. Hash conhecido → só atualiza o
            // caminho, porque o arquivo pode ter sido movido ou renomeado.
            conn.execute(
                "INSERT INTO seen_files (content_hash, path, file_name, size, status)
                 VALUES (?1, ?2, ?3, ?4, 'pending')
                 ON CONFLICT(content_hash) DO UPDATE SET
                   path = excluded.path,
                   file_name = excluded.file_name",
                params![c.content_hash, c.path, c.file_name, c.size],
            )?;
        }
    }

    // Contagem de stubs do iCloud vai pro KV em vez de virar campo de retorno:
    // é um escalar que a Settings lê de vez em quando, não algo que a store de
    // descobertas precise carregar a cada varredura.
    crate::commands::app_settings::set_setting(
        conn,
        ICLOUD_PENDING_KEY,
        &icloud_pending.to_string(),
    )?;

    pending_files(conn)
}

pub fn mark(conn: &Connection, content_hash: &str, status: &str) -> AppResult<()> {
    if !matches!(status, "pending" | "imported" | "ignored" | "invalid") {
        return Err(AppError::Invalid(format!("status inválido '{status}'")));
    }
    conn.execute(
        "UPDATE seen_files
            SET status = ?2,
                resolved_at = CASE WHEN ?2 = 'pending' THEN NULL ELSE datetime('now') END
          WHERE content_hash = ?1",
        params![content_hash, status],
    )?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn scan_watched_folders(db: State<'_, Db>) -> AppResult<Vec<DiscoveredFile>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    scan_all(&conn)
}

#[tauri::command]
#[specta::specta]
pub fn list_pending_files(db: State<'_, Db>) -> AppResult<Vec<DiscoveredFile>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    pending_files(&conn)
}

#[tauri::command]
#[specta::specta]
pub fn mark_file(db: State<'_, Db>, content_hash: String, status: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    mark(&conn, &content_hash, &status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    /// Pasta temporária única por teste, sem depender de crates externas.
    fn tmpdir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("finan-watch-test-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn add_folder_stores_canonical_path_and_label() {
        let conn = fresh_conn();
        let dir = tmpdir("add");
        let folder = add_folder(&conn, dir.to_str().unwrap()).unwrap();

        assert_eq!(folder.label, "finan-watch-test-add");
        assert!(folder.exists);
        assert_eq!(folder.imported_count, 0);
        assert_eq!(folder.last_imported_at, None);
        // canonicalize resolve /var → /private/var no macOS
        assert_eq!(folder.path, std::fs::canonicalize(&dir).unwrap().to_string_lossy());
    }

    #[test]
    fn add_folder_twice_is_idempotent() {
        let conn = fresh_conn();
        let dir = tmpdir("dup");
        let a = add_folder(&conn, dir.to_str().unwrap()).unwrap();
        let b = add_folder(&conn, dir.to_str().unwrap()).unwrap();

        assert_eq!(a.id, b.id, "readicionar devolve a mesma linha");
        assert_eq!(list_folders(&conn).unwrap().len(), 1);
    }

    #[test]
    fn add_folder_rejects_missing_path() {
        let conn = fresh_conn();
        let result = add_folder(&conn, "/caminho/que/nao/existe/mesmo");
        assert!(result.is_err());
    }

    #[test]
    fn add_folder_rejects_file() {
        let conn = fresh_conn();
        let dir = tmpdir("isfile");
        let file = dir.join("extrato.ofx");
        std::fs::write(&file, b"x").unwrap();
        let result = add_folder(&conn, file.to_str().unwrap());
        assert!(result.is_err(), "arquivo não pode ser adicionado como pasta");
    }

    #[test]
    fn missing_folder_reports_exists_false() {
        let conn = fresh_conn();
        let dir = tmpdir("gone");
        add_folder(&conn, dir.to_str().unwrap()).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        let folders = list_folders(&conn).unwrap();
        assert_eq!(folders.len(), 1);
        assert!(!folders[0].exists, "pasta removida deve reportar exists=false");
    }

    #[test]
    fn update_folder_path_preserves_row_id() {
        let conn = fresh_conn();
        let old = tmpdir("old");
        let new = tmpdir("new");
        let created = add_folder(&conn, old.to_str().unwrap()).unwrap();

        let moved = update_folder_path(&conn, created.id, new.to_str().unwrap()).unwrap();
        assert_eq!(moved.id, created.id, "Localizar… preserva o registro");
        assert_eq!(moved.label, "finan-watch-test-new");
        assert_eq!(list_folders(&conn).unwrap().len(), 1);
    }

    #[test]
    fn remove_folder_deletes_row() {
        let conn = fresh_conn();
        let dir = tmpdir("rm");
        let folder = add_folder(&conn, dir.to_str().unwrap()).unwrap();
        remove_folder(&conn, folder.id).unwrap();
        assert!(list_folders(&conn).unwrap().is_empty());
    }

    #[test]
    fn imported_count_reflects_seen_files_in_folder() {
        let conn = fresh_conn();
        let dir = tmpdir("count");
        let folder = add_folder(&conn, dir.to_str().unwrap()).unwrap();

        conn.execute(
            "INSERT INTO seen_files (content_hash, path, file_name, size, status, resolved_at)
             VALUES ('h1', ?1, 'a.ofx', 10, 'imported', '2026-07-02')",
            params![format!("{}/a.ofx", folder.path)],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO seen_files (content_hash, path, file_name, size, status, resolved_at)
             VALUES ('h2', ?1, 'b.ofx', 10, 'ignored', '2026-07-03')",
            params![format!("{}/b.ofx", folder.path)],
        )
        .unwrap();

        let folders = list_folders(&conn).unwrap();
        assert_eq!(folders[0].imported_count, 1, "só 'imported' conta");
        assert_eq!(folders[0].last_imported_at.as_deref(), Some("2026-07-02"));
    }

    #[test]
    fn ensure_dir_creates_and_is_idempotent() {
        let dir = tmpdir("ensure").join("finan");
        assert!(!dir.exists());

        ensure_dir(dir.to_string_lossy().to_string()).unwrap();
        assert!(dir.is_dir());

        // Rodar de novo numa pasta que já existe não pode falhar.
        ensure_dir(dir.to_string_lossy().to_string()).unwrap();
        assert!(dir.is_dir());
    }

    #[test]
    fn dir_exists_distinguishes_directories_from_files() {
        let dir = tmpdir("dir-exists");
        assert!(dir_exists(dir.to_string_lossy().to_string()), "pasta existente");

        let nonexistent = dir.join("nonexistent");
        assert!(!dir_exists(nonexistent.to_string_lossy().to_string()), "caminho inexistente");

        let file = dir.join("test.txt");
        std::fs::write(&file, b"test").unwrap();
        assert!(!dir_exists(file.to_string_lossy().to_string()), "arquivo, não pasta");
    }

    use std::time::{Duration, SystemTime};

    /// Escreve arquivo com mtime "antigo" o suficiente pra não ser adiado.
    fn write_old(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, body).unwrap();
        p
    }

    /// `now` deslocado pro futuro simula um arquivo escrito há bastante tempo,
    /// sem precisar dormir no teste.
    fn later() -> SystemTime {
        SystemTime::now() + Duration::from_secs(60)
    }

    #[test]
    fn scan_finds_ofx_and_ignores_other_extensions() {
        let dir = tmpdir("scan-ext");
        write_old(&dir, "extrato.ofx", "OFXDATA");
        write_old(&dir, "foto.png", "binario");
        write_old(&dir, "notas.txt", "texto");

        let outcome = scan_dir(&dir, later());
        assert_eq!(outcome.candidates.len(), 1);
        assert_eq!(outcome.candidates[0].file_name, "extrato.ofx");
    }

    #[test]
    fn scan_accepts_uppercase_extension() {
        let dir = tmpdir("scan-upper");
        write_old(&dir, "EXTRATO.OFX", "OFXDATA");

        let outcome = scan_dir(&dir, later());
        assert_eq!(outcome.candidates.len(), 1);
    }

    #[test]
    fn scan_hashes_content_so_renames_collide() {
        let dir = tmpdir("scan-hash");
        write_old(&dir, "a.ofx", "MESMO CONTEUDO");
        write_old(&dir, "b.ofx", "MESMO CONTEUDO");

        let outcome = scan_dir(&dir, later());
        assert_eq!(outcome.candidates.len(), 2);
        assert_eq!(
            outcome.candidates[0].content_hash, outcome.candidates[1].content_hash,
            "mesmo conteúdo, mesmo hash — a dedup acontece no INSERT"
        );
    }

    #[test]
    fn scan_defers_files_written_moments_ago() {
        let dir = tmpdir("scan-fresh");
        write_old(&dir, "chegando.ofx", "OFXDATA");

        // `now` = agora: o arquivo tem mtime de milissegundos atrás.
        let outcome = scan_dir(&dir, SystemTime::now());
        assert!(
            outcome.candidates.is_empty(),
            "arquivo ainda sendo escrito (AirDrop/download) deve ser adiado"
        );
    }

    #[test]
    fn scan_reports_icloud_stubs_without_treating_them_as_candidates() {
        let dir = tmpdir("scan-icloud");
        // Placeholder do iCloud: nome com ponto na frente e sufixo .icloud
        write_old(&dir, ".Nubank_2026-07.ofx.icloud", "stub");

        let outcome = scan_dir(&dir, later());
        assert!(outcome.candidates.is_empty());
        assert_eq!(outcome.icloud_pending, 1, "stub vira informação, não erro");
    }

    #[test]
    fn scan_ignores_icloud_stub_of_non_ofx() {
        let dir = tmpdir("scan-icloud-other");
        write_old(&dir, ".ferias.jpg.icloud", "stub");

        let outcome = scan_dir(&dir, later());
        assert_eq!(outcome.icloud_pending, 0);
    }

    #[test]
    fn scan_of_missing_dir_is_empty_not_panic() {
        let dir = tmpdir("scan-gone");
        std::fs::remove_dir_all(&dir).unwrap();

        let outcome = scan_dir(&dir, later());
        assert!(outcome.candidates.is_empty());
        assert_eq!(outcome.icloud_pending, 0);
    }

    #[test]
    fn scan_all_returns_nothing_when_disabled() {
        let conn = fresh_conn();
        let dir = tmpdir("disabled");
        write_old(&dir, "extrato.ofx", "OFXDATA");
        add_folder(&conn, dir.to_str().unwrap()).unwrap();
        // watch_enabled ausente = desligado (default)

        assert!(scan_all(&conn).unwrap().is_empty());
    }

    #[test]
    fn scan_all_registers_new_files_as_pending() {
        let conn = fresh_conn();
        let dir = tmpdir("scan-all");
        write_old(&dir, "extrato.ofx", "OFXDATA");
        add_folder(&conn, dir.to_str().unwrap()).unwrap();
        crate::commands::app_settings::set_setting(&conn, "watch_enabled", "1").unwrap();

        let found = scan_all(&conn).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].file_name, "extrato.ofx");
        assert_eq!(found[0].status, "pending");
    }

    #[test]
    fn scan_all_records_icloud_pending_count() {
        let conn = fresh_conn();
        let dir = tmpdir("icloud-count");
        write_old(&dir, ".Nubank_2026-07.ofx.icloud", "stub");
        add_folder(&conn, dir.to_str().unwrap()).unwrap();
        crate::commands::app_settings::set_setting(&conn, "watch_enabled", "1").unwrap();

        scan_all(&conn).unwrap();

        assert_eq!(
            crate::commands::app_settings::get_setting(&conn, ICLOUD_PENDING_KEY).unwrap(),
            Some("1".to_string())
        );
    }

    #[test]
    fn scan_all_does_not_resurface_resolved_files() {
        let conn = fresh_conn();
        let dir = tmpdir("scan-resolved");
        write_old(&dir, "extrato.ofx", "OFXDATA");
        add_folder(&conn, dir.to_str().unwrap()).unwrap();
        crate::commands::app_settings::set_setting(&conn, "watch_enabled", "1").unwrap();

        let found = scan_all(&conn).unwrap();
        mark(&conn, &found[0].content_hash, "imported").unwrap();

        assert!(
            scan_all(&conn).unwrap().is_empty(),
            "arquivo já resolvido não volta a aparecer"
        );
    }

    #[test]
    fn scan_all_dedups_same_content_under_another_name() {
        let conn = fresh_conn();
        let dir = tmpdir("scan-dedup");
        write_old(&dir, "extrato.ofx", "OFXDATA");
        add_folder(&conn, dir.to_str().unwrap()).unwrap();
        crate::commands::app_settings::set_setting(&conn, "watch_enabled", "1").unwrap();

        let first = scan_all(&conn).unwrap();
        mark(&conn, &first[0].content_hash, "imported").unwrap();

        // Mesmo extrato, baixado de novo com outro nome.
        write_old(&dir, "extrato (1).ofx", "OFXDATA");
        assert!(
            scan_all(&conn).unwrap().is_empty(),
            "hash do conteúdo já foi visto"
        );
    }

    #[test]
    fn scan_all_updates_path_when_file_moves() {
        let conn = fresh_conn();
        let a = tmpdir("move-a");
        let b = tmpdir("move-b");
        write_old(&a, "extrato.ofx", "OFXDATA");
        add_folder(&conn, a.to_str().unwrap()).unwrap();
        add_folder(&conn, b.to_str().unwrap()).unwrap();
        crate::commands::app_settings::set_setting(&conn, "watch_enabled", "1").unwrap();

        let first = scan_all(&conn).unwrap();
        assert_eq!(first.len(), 1);

        std::fs::rename(a.join("extrato.ofx"), b.join("extrato.ofx")).unwrap();
        let second = scan_all(&conn).unwrap();

        assert_eq!(second.len(), 1, "continua um só arquivo pendente");
        assert!(
            second[0].path.contains("move-b"),
            "o caminho é atualizado pra onde o arquivo está agora"
        );
    }

    #[test]
    fn mark_sets_resolved_at() {
        let conn = fresh_conn();
        let dir = tmpdir("mark");
        write_old(&dir, "extrato.ofx", "OFXDATA");
        add_folder(&conn, dir.to_str().unwrap()).unwrap();
        crate::commands::app_settings::set_setting(&conn, "watch_enabled", "1").unwrap();

        let found = scan_all(&conn).unwrap();
        mark(&conn, &found[0].content_hash, "ignored").unwrap();

        let (status, resolved): (String, Option<String>) = conn
            .query_row(
                "SELECT status, resolved_at FROM seen_files WHERE content_hash = ?1",
                params![found[0].content_hash],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(status, "ignored");
        assert!(resolved.is_some());
    }

    #[test]
    fn mark_rejects_unknown_status() {
        let conn = fresh_conn();
        assert!(mark(&conn, "qualquer", "garbage").is_err());
    }

    #[test]
    fn pending_files_lists_only_pending() {
        let conn = fresh_conn();
        let dir = tmpdir("pending-only");
        write_old(&dir, "a.ofx", "CONTEUDO A");
        write_old(&dir, "b.ofx", "CONTEUDO B");
        add_folder(&conn, dir.to_str().unwrap()).unwrap();
        crate::commands::app_settings::set_setting(&conn, "watch_enabled", "1").unwrap();

        let found = scan_all(&conn).unwrap();
        assert_eq!(found.len(), 2);
        mark(&conn, &found[0].content_hash, "invalid").unwrap();

        assert_eq!(pending_files(&conn).unwrap().len(), 1);
    }
}
