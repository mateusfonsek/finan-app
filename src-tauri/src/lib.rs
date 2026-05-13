mod db;
mod error;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let database = db::init(app.handle())
                .expect("failed to initialize database");
            eprintln!("[finan] db at {}", database.path.display());
            app.manage(database);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
