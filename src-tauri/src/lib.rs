mod commands;
mod db;
mod domain;
mod error;

use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::health::health_check,
        commands::accounts::list_accounts,
        commands::accounts::create_or_get_account,
        commands::categories::list_categories,
        commands::categories::create_category,
        commands::transactions::list_transactions,
        commands::transactions::insert_transactions,
        commands::transactions::check_existing_fitids,
        commands::transactions::update_transaction_category,
        commands::transactions::update_transaction_notes,
        commands::rules::list_rules,
        commands::rules::create_rule,
        commands::rules::update_rule,
        commands::rules::delete_rule,
        commands::rules::apply_rules_to_uncategorized,
        commands::summary::summary_kpis,
        commands::summary::summary_by_category,
        commands::summary::summary_by_month,
    ]);

    #[cfg(debug_assertions)]
    {
        let bindings_path = "../src/lib/bindings.ts";
        specta_builder
            .export(
                specta_typescript::Typescript::default()
                    .bigint(specta_typescript::BigIntExportBehavior::Number),
                bindings_path,
            )
            .expect("failed to export TS bindings");

        // Prepend @ts-nocheck so svelte-check ignores unused-local declarations
        // (e.g., Channel/__makeEvents__) that tauri-specta emits even when no events are registered.
        let contents =
            std::fs::read_to_string(bindings_path).expect("failed to read generated bindings");
        if !contents.starts_with("// @ts-nocheck") {
            std::fs::write(bindings_path, format!("// @ts-nocheck\n{contents}"))
                .expect("failed to prepend ts-nocheck to bindings");
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(|app| {
            let database = db::init(app.handle()).expect("failed to initialize database");
            eprintln!("[finan] db at {}", database.path.display());
            app.manage(database);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
