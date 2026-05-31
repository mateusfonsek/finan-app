use std::sync::Mutex;
use tauri::State;

/// Fila de caminhos de arquivos `.ofx` abertos via Finder ("Abrir com finan")
/// que ainda não foram consumidos pelo frontend. Necessária pro cold start: o
/// arquivo chega no `RunEvent::Opened` antes do frontend montar os listeners.
#[derive(Default)]
pub struct PendingOpen(pub Mutex<Vec<String>>);

/// Drena (retorna e limpa) os caminhos pendentes. O frontend chama isso ao
/// montar e a cada evento `open-ofx`.
#[tauri::command]
#[specta::specta]
pub fn take_pending_ofx(pending: State<'_, PendingOpen>) -> Vec<String> {
    let mut queue = pending.0.lock().expect("pending mutex poisoned");
    std::mem::take(&mut *queue)
}
