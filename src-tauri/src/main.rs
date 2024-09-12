// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use color_eyre::eyre::{self, Context};
use github::{
    GetPullRequestResponse, GetRunJobsResponse, GetWorkflowRunsQueryArgs, GetWorkflowRunsResponse,
    GitHubClient, RunJob,
};

mod github;
mod poller_async;

// The workflow id of the tests-ext.yml tests
// TODO: how to calculate progress? Is the list of jobs/steps consistent?
const EXT_TESTS_NUMBER: i64 = 107927392;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value_t = 10)]
    sleep_time: u64,
}

#[derive(Debug, Serialize, Clone, Copy)]
enum Status {
    Queued,
    #[allow(dead_code)]
    InProgress(f32),
    Succeeded,
    Failed,
    Unknown,
}

impl Status {
    fn is_terminal(&self) -> bool {
        matches!(self, Status::Succeeded | Status::Failed)
    }
}

#[derive(Debug, Serialize, Clone)]
struct Pr {
    status: Status,
    number: u64,
    repo: String,
    owner: String,
}

struct PrDescription {
    number: u64,
    repo: String,
    owner: String,
}

impl From<PrDescription> for Pr {
    fn from(value: PrDescription) -> Self {
        Self {
            number: value.number,
            repo: value.repo,
            owner: value.owner,
            status: Status::Unknown,
        }
    }
}

use poller_async::Command;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
// use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};
// use tauri_plugin_positioner::{Position, WindowExt};

// #[tauri::command]
// async fn greet() -> String {
//     "Hello world".to_string()
// }

#[derive(Serialize, Clone)]
struct Event {
    name: String,
}

struct PrList {
    prs: Vec<Pr>,
}

struct AppState {
    poller_handle: poller_async::Handle,
}

#[tauri::command]
fn add_pr(pr_number: u64, state: State<AppState>) {
    tracing::debug!(%pr_number, "got message from frontend");
    let handle = &state.inner().poller_handle;
    handle.add_pr(pr_number);
}

#[tauri::command]
fn clear_prs(state: State<AppState>) {
    tracing::debug!("clearing prs");
    let handle = &state.inner().poller_handle;
    handle.clear_prs();
}

fn main() {
    color_eyre::install().unwrap();
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }

            // set up events
            let app_handle = app.app_handle();

            let handle = poller_async::Handle::new(app_handle);
            let state = AppState {
                poller_handle: handle,
            };
            app.manage(state);

            // let client = GitHubClient::from_env().unwrap();
            // let mut poller = Poller::builder(client)
            //     .with_sleep_time(10)
            //     .with_handle(app_handle)
            //     .build()
            //     .expect("invalid poller configuration");
            // poller.add(PrDescription {
            //     number: 3375,
            //     repo: "localstack-ext".to_string(),
            //     owner: "localstack".to_string(),
            // });
            // let poller_handle = poller.handle();
            // std::thread::spawn(move || poller.start().unwrap());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler!(add_pr, clear_prs))
        .run(tauri::generate_context!())
        .expect("error running application");
}

fn calculate_progress(jobs: &[RunJob]) -> eyre::Result<f32> {
    let mut n_steps_total = 0;
    let mut completed_steps = 0.0f32;
    for job in jobs {
        let n_steps = job.steps.len();

        if job.status == "completed" {
            n_steps_total += n_steps;
            completed_steps += n_steps as f32;
            continue;
        }

        for step in &job.steps {
            n_steps_total += 1;
            if step.status == "completed" {
                completed_steps += 1.0;
            }
        }
    }
    // TODO: fallable cast
    Ok(completed_steps / (n_steps_total as f32))
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use super::calculate_progress;
    use super::GetRunJobsResponse;

    #[test]
    fn from_example() {
        let s = std::fs::read_to_string("testdata/in-progress-jobs.json").unwrap();
        let GetRunJobsResponse { jobs } = serde_json::from_str(&s).unwrap();
        let progress = calculate_progress(&jobs).unwrap();
        assert_abs_diff_eq!(progress, 0.6875, epsilon = 0.001);
    }
}
