mod commands;
mod db;
mod domain;
mod error;

use tauri::menu::{MenuBuilder, SubmenuBuilder};
use tauri::{Emitter, Manager};
use tauri_specta::{collect_commands, Builder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::health::health_check,
        commands::accounts::list_accounts,
        commands::accounts::create_or_get_account,
        commands::categories::list_categories,
        commands::categories::list_categories_with_count,
        commands::categories::create_category,
        commands::categories::update_category,
        commands::categories::delete_category,
        commands::transactions::list_transactions,
        commands::transactions::insert_transactions,
        commands::transactions::check_existing_tx_keys,
        commands::transactions::top_expenses,
        commands::transactions::update_transaction_category,
        commands::transactions::update_transaction_notes,
        commands::rules::list_rules,
        commands::rules::create_rule,
        commands::rules::update_rule,
        commands::rules::delete_rule,
        commands::rules::delete_rule_with_cleanup,
        commands::rules::apply_rules_to_uncategorized,
        commands::rules::calendar_events,
        commands::cnpj::resolve_cnpj,
        commands::suggestions::suggest_rules,
        commands::suggestions::suggest_pattern_for,
        commands::suggestions::auto_classify_with_cnpj,
        commands::summary::summary_kpis,
        commands::summary::summary_by_category,
        commands::summary::summary_by_month,
        commands::summary::investment_summary,
        commands::summary::transfer_summary,
        commands::summary::income_sources,
        commands::backup::db_path,
        commands::backup::read_file_bytes,
        commands::openfile::take_pending_ofx,
        commands::backup::export_backup,
        commands::backup::restore_backup,
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
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .menu(|handle| {
            // App menu (finan): "Sobre" abre nosso modal; "Configurações" sem
            // atalho (decisão do produto). Hide/Quit são os padrões do macOS.
            let app_menu = SubmenuBuilder::new(handle, "finan app")
                .text("about", "Sobre o finan app")
                .separator()
                .text("settings", "Configurações…")
                .separator()
                .services()
                .separator()
                .hide()
                .hide_others()
                .show_all()
                .separator()
                .quit()
                .build()?;

            // Editar: itens padrão fazem os campos de texto da webview se
            // comportarem nativamente (desfazer/recortar/copiar/colar/⌘A).
            let edit_menu = SubmenuBuilder::new(handle, "Editar")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;

            let window_menu = SubmenuBuilder::new(handle, "Janela")
                .minimize()
                .maximize()
                .separator()
                .fullscreen()
                .separator()
                .close_window()
                .build()?;

            let help_menu = SubmenuBuilder::new(handle, "Ajuda")
                .text("github", "finan app no GitHub")
                .build()?;

            MenuBuilder::new(handle)
                .items(&[&app_menu, &edit_menu, &window_menu, &help_menu])
                .build()
        })
        .on_menu_event(|app, event| {
            let id = event.id();
            if id == "about" {
                let _ = app.emit("menu:about", ());
            } else if id == "settings" {
                let _ = app.emit("menu:navigate", "/settings");
            } else if id == "github" {
                let _ = app.emit("menu:github", ());
            }
        })
        .invoke_handler(specta_builder.invoke_handler())
        .setup(|app| {
            let database = db::init(app.handle()).expect("failed to initialize database");
            eprintln!("[finan] db at {}", database.path.display());
            app.manage(database);
            app.manage(commands::openfile::PendingOpen::default());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // Arquivo .ofx aberto via Finder ("Abrir com finan"). Pode chegar no
            // cold start (antes do frontend); guardamos na fila e avisamos o front.
            if let tauri::RunEvent::Opened { urls } = event {
                let paths: Vec<String> = urls
                    .iter()
                    .filter_map(|u| u.to_file_path().ok())
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                if !paths.is_empty() {
                    if let Some(pending) = app_handle.try_state::<commands::openfile::PendingOpen>() {
                        pending
                            .0
                            .lock()
                            .expect("pending mutex poisoned")
                            .extend(paths);
                    }
                    let _ = app_handle.emit("open-ofx", ());
                }
            }
        });
}
