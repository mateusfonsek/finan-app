mod commands;
mod db;
mod domain;
mod error;
mod locale;

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
        locale::list_locales,
        locale::get_active_locale,
        locale::set_active_locale,
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
            let locale_state = locale::LocaleState::init(app.handle());

            // Native menu, localized from the active pack. Built here (not via
            // `.menu()`) because the path resolver used to load the pack isn't
            // ready until setup. The event handler is registered on the builder
            // above and fires regardless of when the menu is created.
            {
                let handle = app.handle().clone();
                let pack = locale_state.pack.lock().expect("locale mutex poisoned");
                let app_menu = SubmenuBuilder::new(&handle, pack.menu_str("app_name", "finan app"))
                    .text("about", pack.menu_str("about", "Sobre o finan app"))
                    .separator()
                    .text("settings", pack.menu_str("settings", "Configurações…"))
                    .separator()
                    .services()
                    .separator()
                    .hide()
                    .hide_others()
                    .show_all()
                    .separator()
                    .quit()
                    .build()?;
                let edit_menu = SubmenuBuilder::new(&handle, pack.menu_str("edit", "Editar"))
                    .undo()
                    .redo()
                    .separator()
                    .cut()
                    .copy()
                    .paste()
                    .select_all()
                    .build()?;
                let window_menu = SubmenuBuilder::new(&handle, pack.menu_str("window", "Janela"))
                    .minimize()
                    .maximize()
                    .separator()
                    .fullscreen()
                    .separator()
                    .close_window()
                    .build()?;
                let help_menu = SubmenuBuilder::new(&handle, pack.menu_str("help", "Ajuda"))
                    .text("github", pack.menu_str("github", "finan app no GitHub"))
                    .build()?;
                let menu = MenuBuilder::new(&handle)
                    .items(&[&app_menu, &edit_menu, &window_menu, &help_menu])
                    .build()?;
                app.set_menu(menu)?;
            }

            let database = db::init(app.handle()).expect("failed to initialize database");
            eprintln!("[finan] db at {}", database.path.display());
            // Only a brand-new DB is seeded from the active locale pack — never
            // an existing user's DB (would clobber their renamed categories).
            if database.fresh {
                let conn = database.conn.lock().expect("db mutex poisoned");
                let pack = locale_state.pack.lock().expect("locale mutex poisoned");
                db::seed_from_pack(&conn, &pack).expect("failed to seed from locale pack");
            }
            app.manage(database);
            app.manage(locale_state);
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
