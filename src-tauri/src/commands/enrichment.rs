//! Settings surface for tax-id enrichment.

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::db::Db;
use crate::enrich;
use crate::error::AppResult;
use crate::locale::LocaleState;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EnrichmentStatus {
    /// Active locale has a tax-id format and a known provider. When `false`
    /// the UI hides the setting instead of showing an inert switch.
    pub available: bool,
    pub enabled: bool,
    /// e.g. "CNPJ" — so the screen names the id the way the user knows it.
    pub tax_id_name: String,
    /// e.g. "brasilapi" — shown before opting in, so the user knows who is
    /// being called.
    pub provider: String,
}

#[tauri::command]
#[specta::specta]
pub fn enrichment_status(
    db: State<'_, Db>,
    locale: State<'_, LocaleState>,
) -> AppResult<EnrichmentStatus> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let pack = locale.pack.lock().expect("locale mutex poisoned");
    Ok(EnrichmentStatus {
        available: enrich::is_available(&pack),
        enabled: enrich::is_enabled(&conn),
        tax_id_name: pack.manifest.tax_id.name.clone(),
        provider: pack.manifest.tax_id.provider.clone(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn set_enrichment_enabled(db: State<'_, Db>, enabled: bool) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![enrich::ENABLED_KEY, if enabled { "1" } else { "0" }],
    )?;
    Ok(())
}
