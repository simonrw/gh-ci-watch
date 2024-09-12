use std::time::Duration;

use color_eyre::eyre::{self, Context};
use tauri::async_runtime::{self, channel, JoinHandle, Receiver, Sender};
use tauri::{AppHandle, Manager};
use tokio::task::JoinSet;
use tokio::time;
use tracing::Instrument;

use crate::github::{
    GetPullRequestResponse, GetRunJobsResponse, GetWorkflowRunsQueryArgs, GetWorkflowRunsResponse,
    GitHubClient,
};
use crate::{calculate_progress, Pr, Status, EXT_TESTS_NUMBER};

static OWNER: &str = "localstack";
static REPO: &str = "localstack-ext";

#[derive(Debug, Clone, Copy)]
pub enum Command {
    AddPr(u64),
    RemovePr(u64),
    ClearPrs,
    Tick,
}

pub struct Poller {
    receiver: Receiver<Command>,
    prs: Vec<u64>,
    handle: AppHandle,
    heartbeat_handle: JoinHandle<()>,
    internal_queue: Sender<Command>,
    client: GitHubClient,
}

async fn heartbeat(tx: Sender<Command>) {
    let mut interval = time::interval(Duration::from_secs(10));

    loop {
        interval.tick().await;
        let _ = tx.send(Command::Tick).await;
    }
}

impl Poller {
    pub fn new(
        internal_queue: Sender<Command>,
        receiver: Receiver<Command>,
        handle: AppHandle,
    ) -> Self {
        let heartbeat_queue = internal_queue.clone();
        let heartbeat_handle = async_runtime::spawn(async move {
            heartbeat(heartbeat_queue).await;
        });

        let client = GitHubClient::from_env().expect("creating GitHub client");

        Self {
            receiver,
            prs: Vec::new(),
            handle,
            heartbeat_handle,
            internal_queue,
            client,
        }
    }

    async fn handle_command(&mut self, cmd: Command) -> eyre::Result<()> {
        match cmd {
            Command::AddPr(number) => {
                self.prs.push(number);
                self.tick().await;
            }
            Command::RemovePr(number) => {
                // TODO: remove the clone
                self.prs = self
                    .prs
                    .clone()
                    .into_iter()
                    .filter(|pr| *pr != number)
                    .collect();
                self.tick().await;
            }
            Command::ClearPrs => {
                self.prs.clear();
                self.tick().await;
            }
            Command::Tick => {
                tracing::debug!("got heartbeat tick");

                let pr_results = self
                    .update_pr_statuses()
                    .await
                    .wrap_err("error updating PR statuses")?;
                self.handle
                    .emit_all("state", pr_results)
                    .wrap_err("sending message to frontend")?;
            }
        }
        Ok(())
    }

    async fn tick(&self) {
        let _ = self.internal_queue.send(Command::Tick).await;
    }

    async fn update_pr_statuses(&self) -> eyre::Result<Vec<Pr>> {
        tracing::debug!("updating PR statuses");
        let mut set = JoinSet::new();

        for pr_number in self.prs.clone() {
            let client = self.client.clone();
            let span = tracing::debug_span!("fetch task", %pr_number);
            let _abort = set.spawn(async move {
                tracing::debug!("fetching pr info");
                let pr_info: GetPullRequestResponse = client
                    .get(
                        format!(
                            "https://api.github.com/repos/{}/{}/pulls/{}",
                            OWNER, REPO, pr_number,
                        ),
                        None::<()>,
                    )
                    .await
                    .wrap_err("fetching branch info")?;

                // fetch workflow runs for branch
                tracing::debug!("fetching workflow runs");
                let GetWorkflowRunsResponse { mut workflow_runs } = client
                    .get(
                        format!(
                            "https://api.github.com/repos/{}/{}/actions/workflows/{}/runs",
                            OWNER, REPO, EXT_TESTS_NUMBER
                        ),
                        Some(GetWorkflowRunsQueryArgs {
                            branch: pr_info.head.branch.clone(),
                        }),
                    )
                    .await
                    .wrap_err("fetching workflow runs")?;
                workflow_runs.sort_by_key(|k| k.run_number);
                let Some(run) = workflow_runs.pop() else {
                    // TODO
                    eyre::bail!("no workflow runs found");
                };

                tracing::debug!(run_id = %run.id, "got latest run");

                // DEBUG
                // let mut f = std::fs::File::create("in-progress-jobs.json").unwrap();
                // if let Err(e) = serde_json::to_writer_pretty(&mut f, &jobs) {
                //     tracing::warn!(error = ?e, "error saving in-progress job JSON state");
                // }

                tracing::debug!("updating PR state");
                match run.status.as_str() {
                    "completed" => match run.conclusion.as_deref() {
                        Some("failure") => {
                            Ok(Pr {
                                status: Status::Failed,
                                number: pr_number,
                                repo: REPO.to_string(),
                                owner: OWNER.to_string(),
                            })
                            // tracing::debug!(before = ?pr.status, after = ?Status::Failed, "updating status");
                            // pr.status = Status::Failed;
                        }
                        Some("success") => Ok(Pr {
                            status: Status::Succeeded,
                            number: pr_number,
                            repo: REPO.to_string(),
                            owner: OWNER.to_string(),
                        }),
                        other => todo!(
                            "unhandled combination of status: completed and conclusion: {other:?}"
                        ),
                    },
                    "queued" => Ok(Pr {
                        status: Status::Queued,
                        number: pr_number,
                        repo: REPO.to_string(),
                        owner: OWNER.to_string(),
                    }),
                    "in_progress" => {
                        // get run jobs
                        tracing::debug!("fetching jobs for run");
                        let GetRunJobsResponse { jobs } = client
                            .get(
                                format!(
                                    "https://api.github.com/repos/{}/{}/actions/runs/{}/jobs",
                                    OWNER, REPO, run.id
                                ),
                                None::<()>,
                            )
                            .await
                            .wrap_err("fetching run jobs")?;

                        let progress = calculate_progress(&jobs).unwrap_or(0.0);
                        let status = Status::InProgress(progress);
                        Ok(Pr {
                            status,
                            number: pr_number,
                            repo: REPO.to_string(),
                            owner: OWNER.to_string(),
                        })
                    }
                    other => todo!("unhandled status: {other}"),
                }
                // tracing::debug!("finished");
                // Ok(())
            });
        }

        let mut pr_details = Vec::new();

        while let Some(res) = set.join_next().await {
            match res {
                Ok(Ok(pr)) => pr_details.push(pr),
                Ok(Err(e)) => tracing::error!(error = %e, "error fetching pr information"),
                Err(e) => tracing::warn!(error = %e, "joining fetch future"),
            }
        }

        Ok(pr_details)
    }
}

#[derive(Clone)]
pub struct Handle {
    sender: Sender<Command>,
}

impl Handle {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        let (sender, receiver) = channel(100);
        let heartbeat_handle = sender.clone();
        let poller = Poller::new(heartbeat_handle, receiver, app_handle);
        async_runtime::spawn(run_poller(poller));
        Self { sender }
    }

    pub fn add_pr(&self, pr_number: u64) {
        let cmd = Command::AddPr(pr_number);
        if let Err(e) = self.sender.blocking_send(cmd) {
            tracing::warn!(error = %e, pr = %pr_number, "poller is down");
        }
    }

    pub fn remove_pr(&self, pr_number: u64) {
        let cmd = Command::RemovePr(pr_number);
        if let Err(e) = self.sender.blocking_send(cmd) {
            tracing::warn!(error = %e, pr = %pr_number, "poller is down");
        }
    }

    pub fn clear_prs(&self) {
        let cmd = Command::ClearPrs;
        if let Err(e) = self.sender.blocking_send(cmd) {
            tracing::warn!(error = %e, "poller is down");
        }
    }
}

pub async fn run_poller(mut poller: Poller) {
    while let Some(cmd) = poller.receiver.recv().await {
        if let Err(e) = poller.handle_command(cmd).await {
            tracing::error!(error = %e, command = ?cmd, "handling command");
        }
    }
}
