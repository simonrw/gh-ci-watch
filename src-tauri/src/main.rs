use color_eyre::eyre::{self, Context};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, USER_AGENT},
    IntoUrl,
};
use serde::{Deserialize, Serialize};

// TODO: how to calculate progress? Is the list of jobs/steps consistent?
const EXT_TESTS_NUMBER: i64 = 107927392;

#[derive(Debug, Clone, Copy)]
enum Status {
    Queued,
    InProgress(f32),
    Succeeded,
    Failed,
    Unknown,
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

#[derive(Debug, Deserialize)]
struct Commit {
    id: String,
    message: String,
    // TODO: datetime
    timestamp: String,
}

struct Poller {
    client: GitHubClient,
    prs: Vec<Pr>,
}

impl Poller {
    fn new(client: GitHubClient) -> Self {
        // TODO: enumerate workflows on startup and use cache?
        Self {
            client,
            prs: Vec::new(),
        }
    }

    fn add(&mut self, pr: impl Into<Pr>) {
        self.prs.push(pr.into());
    }

    fn start(&mut self) -> eyre::Result<()> {
        for pr in &mut self.prs {
            let span = tracing::debug_span!("", pr = ?pr.number);
            let _guard = span.enter();

            // fetch pr branch
            tracing::debug!("fetching pr info");
            let pr_info = {
                #[derive(Debug, Deserialize)]
                struct Head {
                    #[serde(rename = "ref")]
                    branch: String,
                }
                #[derive(Debug, Deserialize)]
                struct PrInfo {
                    title: String,
                    head: Head,
                }

                let pr_info: PrInfo = self
                    .client
                    .get(
                        format!(
                            "https://api.github.com/repos/{}/{}/pulls/{}",
                            &pr.owner, &pr.repo, &pr.number
                        ),
                        None::<()>,
                    )
                    .wrap_err("fetching branch info")?;
                pr_info
            };

            // fetch workflow runs for branch
            tracing::debug!("fetching workflow runs");
            let mut workflow_runs = {
                #[derive(Debug, Deserialize)]
                struct Run {
                    id: u64,
                    name: String,
                    status: String,
                    conclusion: Option<String>,
                    display_title: String,
                    run_attempt: u64,
                    run_number: u64,
                    // TODO: datetime
                    run_started_at: String,
                    head_commit: Commit,
                }

                #[derive(Debug, Deserialize)]
                struct Response {
                    workflow_runs: Vec<Run>,
                }

                #[derive(Serialize)]
                struct Query {
                    branch: String,
                }

                let workflow_runs: Response = self
                    .client
                    .get(
                        format!(
                            "https://api.github.com/repos/{}/{}/actions/workflows/{}/runs",
                            &pr.owner, &pr.repo, EXT_TESTS_NUMBER
                        ),
                        Some(Query {
                            branch: pr_info.head.branch.clone(),
                        }),
                    )
                    .wrap_err("fetching workflow runs")?;
                workflow_runs.workflow_runs
            };

            workflow_runs.sort_by_key(|k| k.run_number);
            let Some(run) = workflow_runs.pop() else {
                // TODO
                eyre::bail!("no workflow runs found");
            };

            tracing::debug!(run_id = %run.id, "got latest run");

            // TODO: only if the run is in progress
            // get run jobs
            tracing::debug!("fetching jobs for run");
            let jobs = {
                #[derive(Debug, Deserialize)]
                struct Step {
                    name: String,
                    status: String,
                    conclusion: Option<String>,
                    started_at: Option<String>,
                    completed_at: Option<String>,
                }

                #[derive(Debug, Deserialize)]
                struct Job {
                    id: u64,
                    name: String,
                    status: String,
                    conclusion: Option<String>,
                    started_at: String,
                    completed_at: Option<String>,
                    steps: Vec<Step>,
                }
                #[derive(Debug, Deserialize)]
                struct Response {
                    jobs: Vec<Job>,
                }

                let jobs: Response = self
                    .client
                    .get(
                        format!(
                            "https://api.github.com/repos/{}/{}/actions/runs/{}/jobs",
                            pr.owner, pr.repo, run.id
                        ),
                        None::<()>,
                    )
                    .wrap_err("fetching run jobs")?;
                jobs
            };

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
                    let new_status = Status::InProgress(0.0);
                    tracing::debug!(before = ?pr.status, after = ?new_status, "updating status");
                    pr.status = new_status;
                }
                other => todo!("unhandled status: {other}"),
            }
            tracing::debug!("finished");
        }

        std::process::exit(0);

        // std::thread::sleep(Duration::from_secs(1));
    }
}

struct GitHubClient {
    client: Client,
    token: String,
}

// Constructors
impl GitHubClient {
    fn from_env() -> eyre::Result<Self> {
        let token = std::env::var("GITHUB_TOKEN").wrap_err("no GitHub token found")?;

        let mut headers = HeaderMap::new();
        headers.append(USER_AGENT, HeaderValue::from_static("gh-ci-watch"));

        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .wrap_err("constructing HTTP client")?;
        Ok(Self { client, token })
    }

    fn get<T, Q>(&self, url: impl IntoUrl, query: Option<Q>) -> eyre::Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
        Q: Serialize,
    {
        let url = url.into_url().wrap_err("invalid URL")?;
        let span = tracing::debug_span!("", url=%url);
        let _guard = span.enter();

        let mut builder = self.client.get(url).bearer_auth(&self.token);
        if let Some(query) = &query {
            builder = builder.query(query);
        }

        tracing::debug!("sending http request");
        let response = builder.send().wrap_err("sending GET request")?;
        if let Err(e) = response.error_for_status_ref() {
            tracing::warn!(error = %e, "bad status from GitHub");
            eyre::bail!("bad error status: {e}");
        }
        tracing::debug!("got http response");
        response.json().wrap_err("decoding JSON response")
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install().unwrap();
    tracing_subscriber::fmt::init();

    let client = GitHubClient::from_env().unwrap();
    let mut poller = Poller::new(client);
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
