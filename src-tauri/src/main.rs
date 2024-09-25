// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::{hash_map::Entry, HashMap};

use color_eyre::eyre::{self, Context};

mod config;
mod fetcher;
mod github;

use config::AppConfig;
use fetcher::{Fetcher, Pr};
use github::WorkflowDetails;
use sentry::ClientInitGuard;
use tauri::State;

#[cfg(debug_assertions)]
use tauri::Manager;
use tokio::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WorkflowCacheKey {
    repo: String,
    owner: String,
}

struct AppState {
    fetcher: Fetcher,
    workflow_cache: Mutex<HashMap<WorkflowCacheKey, Vec<WorkflowDetails>>>,
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

    let cache_key = WorkflowCacheKey {
        repo: repo.clone(),
        owner: owner.clone(),
    };

    let mut cache = state.workflow_cache.lock().await;

    // TODO: better entry API
    match cache.entry(cache_key.clone()) {
        Entry::Occupied(entry) => {
            tracing::trace!(?cache_key, "workflow cache hit");
            Ok(entry.get().clone())
        }
        Entry::Vacant(vacant_entry) => {
            tracing::trace!(?cache_key, "workflow cache miss");
            let fetcher = &state.fetcher;
            let workflows = fetcher
                .fetch_workflows(token, &owner, &repo)
                .await
                .map_err(|e| {
                    tracing::warn!(error = %e, %owner, %repo, "error fetching workflows");
                    format!("Error fetching workflows for '{owner}/{repo}': {e}")
                })?;
            vacant_entry.insert(workflows.clone());
            Ok(workflows)
        }
    }
}

fn create_app<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
    base_url: impl Into<String>,
) -> eyre::Result<tauri::App<R>> {
    let fetcher = Fetcher::new(base_url);
    let app_state = AppState {
        fetcher,
        workflow_cache: Default::default(),
    };

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

fn init_sentry(enable: bool) -> Option<ClientInitGuard> {
    if enable {
        tracing::debug!("enabling sentry integration for error reporting");

        let options = sentry::ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: 1.0,
            ..Default::default()
        };
        let guard = sentry::init((
            "https://f4117328b0e50349c718cd9a952f31f3@o366030.ingest.us.sentry.io/4508015566848000",
            options,
        ));
        Some(guard)
    } else {
        None
    }
}

fn main() {
    color_eyre::install().unwrap();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(sentry::integrations::tracing::layer())
        .init();

    let config = AppConfig::from_default_path().unwrap_or_default();
    tracing::debug!(?config, "loaded config");

    let _sentry_guard = init_sentry(config.enable_sentry);

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
