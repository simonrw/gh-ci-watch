// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use color_eyre::eyre::{self, Context};

mod fetcher;
mod github;

use fetcher::{Fetcher, Pr};
use github::WorkflowDetails;
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
    workflow_id: u64,
    pr_number: u64,
    token: String,
    state: State<'_, AppState>,
) -> Result<Pr, String> {
    tracing::debug!(%owner, %repo, %workflow_id, %pr_number, "requesting status");
    let fetcher = &state.fetcher;
    let pr = fetcher
        .fetch(token, owner, repo, workflow_id, pr_number)
        .await
        .map_err(|e| format!("Error fetching pr status: {e}"))?;
    tracing::debug!(?pr, "got pr");
    Ok(pr)
}

#[tauri::command]
async fn fetch_workflows_for_repo(
    owner: String,
    repo: String,
    token: String,
    state: State<'_, AppState>,
) -> Result<Vec<WorkflowDetails>, String> {
    tracing::debug!(%owner, %repo, "requesting workflows for repo");

    let fetcher = &state.fetcher;
    let workflows = fetcher
        .fetch_workflows(token, &owner, &repo)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, %owner, %repo, "error fetching workflows");
            format!("Error fetching workflows for '{owner}/{repo}': {e}")
        })?;
    tracing::debug!(?workflows, "got workflows");
    Ok(workflows)
}

fn create_app<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
    base_url: impl Into<String>,
) -> eyre::Result<tauri::App<R>> {
    let fetcher = Fetcher::new(base_url);
    let app_state = AppState { fetcher };

    builder
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler!(
            fetch_status,
            fetch_workflows_for_repo
        ))
        .build(tauri::generate_context!())
        .wrap_err("building tauri application")
}

fn main() {
    color_eyre::install().unwrap();
    tracing_subscriber::fmt::init();

    let app = create_app(tauri::Builder::default(), "https://api.github.com").unwrap();
    app.run(|_app_handle, _event| {});
}

#[cfg(test)]
mod tests {
    use color_eyre::eyre::{self, Context};
    use httpmock::prelude::*;
    use tauri::Manager;

    use crate::{
        create_app,
        github::{GetWorkflowsResponse, WorkflowDetails},
    };

    fn init_logger() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[test]
    fn fetch_workflows() {
        init_logger();

        let server = MockServer::start();

        let mock_workflows = vec![WorkflowDetails {
            id: 10,
            name: "my-workflow".into(),
            filename: ".github/workflows/test.yml".into(),
        }];
        let response = GetWorkflowsResponse {
            workflows: mock_workflows.clone(),
        };
        let workflows_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/repos/localstack/localstack/actions/workflows");
            then.status(200)
                .header("content-type", "application/json")
                .body(serde_json::to_vec(&response).unwrap());
        });

        let app =
            create_app(tauri::test::mock_builder(), server.base_url()).expect("creating mock app");
        let window = app.get_window("main").unwrap();

        tauri::test::assert_ipc_response(
            &window,
            tauri::InvokePayload {
                cmd: "fetch_workflows_for_repo".into(),
                tauri_module: None,
                invoke_key: Some(tauri::test::INVOKE_KEY.into()),
                callback: tauri::api::ipc::CallbackFn(0),
                error: tauri::api::ipc::CallbackFn(1),
                inner: serde_json::json!( {
                    "owner": "localstack",
                    "repo": "localstack",
                    "token": "abc123",
                }),
            },
            Ok(mock_workflows),
        );

        workflows_mock.assert();
    }
}
