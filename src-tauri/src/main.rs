// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod fetcher;
mod github;

use fetcher::{Fetcher, Pr};
use tauri::State;

#[cfg(debug_assertions)]
use tauri::Manager;

struct AppState {
    fetcher: Fetcher,
}

#[tauri::command]
async fn fetch_status(
    owner: String,
    repo: String,
    pr_number: u64,
    token: String,
    state: State<'_, AppState>,
) -> Result<Pr, String> {
    tracing::debug!(%owner, %repo, %pr_number, "requesting status");
    let fetcher = &state.fetcher;
    let pr = fetcher
        .fetch(token, owner, repo, pr_number)
        .await
        .map_err(|e| format!("Error fetching pr status: {e}"))?;
    tracing::debug!(?pr, "got pr");
    Ok(pr)
}

fn main() {
    color_eyre::install().unwrap();
    tracing_subscriber::fmt::init();

    let fetcher = Fetcher::new();
    let app_state = AppState { fetcher };

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler!(fetch_status))
        .run(tauri::generate_context!())
        .expect("error running application");
}
