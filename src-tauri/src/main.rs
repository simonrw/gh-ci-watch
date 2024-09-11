use std::time::Duration;

use clap::Parser;
use color_eyre::eyre::{self, Context};
use github::{
    GetPullRequestResponse, GetRunJobsResponse, GetWorkflowRunsQueryArgs, GetWorkflowRunsResponse,
    GitHubClient, RunJob,
};

mod github;

// The workflow id of the tests-ext.yml tests
// TODO: how to calculate progress? Is the list of jobs/steps consistent?
const EXT_TESTS_NUMBER: i64 = 107927392;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value_t = 10)]
    sleep_time: u64,
}

#[derive(Debug, Clone, Copy)]
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
        match self {
            Status::Succeeded | Status::Failed => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
struct Pr {
    status: Status,
    number: i64,
    repo: String,
    owner: String,
}

struct PrDescription {
    number: i64,
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

struct PollerBuilder {
    client: GitHubClient,
    sleep_time: Option<u64>,
}

impl PollerBuilder {
    fn with_sleep_time(mut self, sleep_time: u64) -> Self {
        self.sleep_time = Some(sleep_time);
        self
    }

    fn build(self) -> eyre::Result<Poller> {
        let sleep_time = self
            .sleep_time
            .ok_or(eyre::eyre!("no sleep time provided"))?;
        Ok(Poller {
            client: self.client,
            prs: Vec::new(),
            sleep_time: Duration::from_secs(sleep_time),
        })
    }
}

struct Poller {
    client: GitHubClient,
    prs: Vec<Pr>,
    sleep_time: Duration,
}

impl Poller {
    fn builder(client: GitHubClient) -> PollerBuilder {
        // TODO: enumerate workflows on startup and use cache?
        PollerBuilder {
            client,
            sleep_time: None,
        }
    }

    fn add(&mut self, pr: impl Into<Pr>) {
        self.prs.push(pr.into());
    }

    fn start(&mut self) -> eyre::Result<()> {
        loop {
            for pr in &mut self.prs {
                let span = tracing::debug_span!("", pr = ?pr.number);
                let _guard = span.enter();

                if pr.status.is_terminal() {
                    tracing::debug!("skipping finished job");
                    continue;
                }

                // fetch pr branch
                tracing::debug!("fetching pr info");
                let pr_info: GetPullRequestResponse = self
                    .client
                    .get(
                        format!(
                            "https://api.github.com/repos/{}/{}/pulls/{}",
                            &pr.owner, &pr.repo, &pr.number
                        ),
                        None::<()>,
                    )
                    .wrap_err("fetching branch info")?;

                // fetch workflow runs for branch
                tracing::debug!("fetching workflow runs");
                let GetWorkflowRunsResponse { mut workflow_runs } = self
                    .client
                    .get(
                        format!(
                            "https://api.github.com/repos/{}/{}/actions/workflows/{}/runs",
                            &pr.owner, &pr.repo, EXT_TESTS_NUMBER
                        ),
                        Some(GetWorkflowRunsQueryArgs {
                            branch: pr_info.head.branch.clone(),
                        }),
                    )
                    .wrap_err("fetching workflow runs")?;
                workflow_runs.sort_by_key(|k| k.run_number);
                let Some(run) = workflow_runs.pop() else {
                    // TODO
                    eyre::bail!("no workflow runs found");
                };

                tracing::debug!(run_id = %run.id, "got latest run");

                // TODO: only if the run is in progress
                // get run jobs
                tracing::debug!("fetching jobs for run");
                let GetRunJobsResponse { jobs } = self
                    .client
                    .get(
                        format!(
                            "https://api.github.com/repos/{}/{}/actions/runs/{}/jobs",
                            pr.owner, pr.repo, run.id
                        ),
                        None::<()>,
                    )
                    .wrap_err("fetching run jobs")?;

                // DEBUG
                // let mut f = std::fs::File::create("in-progress-jobs.json").unwrap();
                // if let Err(e) = serde_json::to_writer_pretty(&mut f, &jobs) {
                //     tracing::warn!(error = ?e, "error saving in-progress job JSON state");
                // }

                tracing::debug!("updating PR state");
                match run.status.as_str() {
                    "completed" => match run.conclusion.as_deref() {
                        Some("failure") => {
                            tracing::debug!(before = ?pr.status, after = ?Status::Failed, "updating status");
                            pr.status = Status::Failed;
                        }
                        Some("success") => {
                            tracing::debug!(before = ?pr.status, after = ?Status::Succeeded, "updating status");
                            pr.status = Status::Succeeded;
                        }
                        other => todo!(
                            "unhandled combination of status: completed and conclusion: {other:?}"
                        ),
                    },
                    "queued" => {
                        let new_status = Status::Queued;
                        tracing::debug!(before = ?pr.status, after = ?new_status, "updating status");
                        pr.status = new_status;
                    }
                    "in_progress" => {
                        // TODO: work out completion percentage
                        let progress = calculate_progress(&jobs).unwrap_or(0.0);
                        let new_status = Status::InProgress(progress);
                        tracing::debug!(before = ?pr.status, after = ?new_status, "updating status");
                        pr.status = new_status;
                    }
                    other => todo!("unhandled status: {other}"),
                }
                tracing::debug!("finished");
            }
            std::thread::sleep(self.sleep_time);
        }
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install().unwrap();
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let client = GitHubClient::from_env().unwrap();
    let mut poller = Poller::builder(client)
        .with_sleep_time(args.sleep_time)
        .build()
        .wrap_err("invalid poller configuration")?;
    poller.add(PrDescription {
        number: 3375,
        repo: "localstack-ext".to_string(),
        owner: "localstack".to_string(),
    });

    poller.start().wrap_err("running poller")?;

    Ok(())
}

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};
// use tauri_plugin_positioner::{Position, WindowExt};

// #[tauri::command]
// async fn greet() -> String {
//     "Hello world".to_string()
// }

// #[allow(unused)]
// fn tauri_main() {
//     let github_token = std::env::var("GITHUB_TOKEN")
//         .expect("no GITHUB_TOKEN found in environment; cannot continue");

//     let quit = CustomMenuItem::new("quit".to_string(), "Quit").accelerator("Cmd+Q");
//     let system_tray_menu = SystemTrayMenu::new().add_item(quit);

//     tauri::Builder::default()
//         .plugin(tauri_plugin_positioner::init())
//         .system_tray(SystemTray::new().with_menu(system_tray_menu))
//         .on_system_tray_event(|app, event| {
//             tauri_plugin_positioner::on_tray_event(app, &event);
//             match event {
//                 SystemTrayEvent::LeftClick { .. } => {
//                     let window = app.get_window("main").unwrap();
//                     let _ = window.move_window(Position::TrayCenter);
//                     if window.is_visible().unwrap() {
//                         window.hide().unwrap();
//                     } else {
//                         window.show().unwrap();
//                         window.set_focus().unwrap();
//                     }
//                 }
//                 SystemTrayEvent::MenuItemClick { id, .. } => {
//                     if id.as_str() == "quit" {
//                         std::process::exit(0)
//                     }
//                 }
//                 _ => {}
//             }
//         })
//         .on_window_event(|event| {
//             if let tauri::WindowEvent::Focused(is_focused) = event.event() {
//                 if !is_focused {
//                     event.window().hide().unwrap();
//                 }
//             }
//         })
//         .setup(|app| {
//             app.set_activation_policy(tauri::ActivationPolicy::Accessory);
//             Ok(())
//         })
//         .invoke_handler(tauri::generate_handler![greet])
//         .run(tauri::generate_context!())
//         .expect("error while running tauri application");
// }

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
