use std::sync::Arc;

use crate::{
    config::AppConfig,
    github::{
        GetPullRequestResponse, GetRunJobsResponse, GetWorkflowRunsQueryArgs,
        GetWorkflowRunsResponse, GetWorkflowsResponse, GitHubClient, RunJob, WorkflowDetails,
    },
};
use color_eyre::eyre::{self, Context};
use serde::Serialize;

pub struct Fetcher {
    client: GitHubClient,
}

impl Fetcher {
    pub fn new(base_url: impl Into<String>, app_config: Arc<AppConfig>) -> Self {
        let client = GitHubClient::new(base_url, app_config);
        Self { client }
    }

    pub async fn fetch(
        &self,
        token: impl AsRef<str>,
        owner: impl AsRef<str>,
        repo: impl AsRef<str>,
        workflow_id: u64,
        pr_number: u64,
    ) -> eyre::Result<Pr> {
        let token = token.as_ref();
        let owner = owner.as_ref();
        let repo = repo.as_ref();
        let pr_info = self.fetch_pr_info(owner, repo, pr_number, token).await?;

        // fetch workflow runs for branch
        let GetWorkflowRunsResponse { mut workflow_runs } = self
            .fetch_workflow_runs(owner, repo, workflow_id, pr_info.head.branch, token)
            .await?;
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
        // get run jobs
        tracing::debug!("fetching jobs for run");
        let GetRunJobsResponse { jobs } = self
            .fetch_run_jobs(owner, repo, run.id, token)
            .await
            .wrap_err("fetching run jobs")?;

        let ProgressResult {
            progress,
            complete,
            total,
        } = calculate_progress(&jobs);

        tracing::debug!("updating PR state");
        let pr_result = match run.status.as_str() {
            "completed" => match run.conclusion.as_deref() {
                Some("failure") => {
                    Pr {
                        status: Status::Failed,
                        title: pr_info.title,
                        description: pr_info.description.unwrap_or_else(String::new),
                        num_steps: total,
                        num_complete_steps: complete,
                        pr_url: pr_info.url,
                        run_url: run.url,
                    }
                    // tracing::debug!(before = ?pr.status, after = ?Status::Failed, "updating status");
                    // pr.status = Status::Failed;
                }
                Some("success") => Pr {
                    status: Status::Succeeded,
                    title: pr_info.title,
                    description: pr_info.description.unwrap_or_default(),
                    num_steps: total,
                    num_complete_steps: complete,
                    pr_url: pr_info.url,
                    run_url: run.url,
                },
                other => {
                    todo!("unhandled combination of status: completed and conclusion: {other:?}")
                }
            },
            "queued" => Pr {
                status: Status::Queued,
                title: pr_info.title,
                description: pr_info.description.unwrap_or_default(),
                num_steps: total,
                num_complete_steps: complete,
                pr_url: pr_info.url,
                run_url: run.url,
            },
            "in_progress" => {
                // get run jobs
                let status = Status::InProgress(progress);
                Pr {
                    status,
                    title: pr_info.title,
                    description: pr_info.description.unwrap_or_default(),
                    num_steps: total,
                    num_complete_steps: complete,
                    pr_url: pr_info.url,
                    run_url: run.url,
                }
            }
            "pending" => Pr {
                status: Status::Queued,
                title: pr_info.title,
                description: pr_info.description.unwrap_or_default(),
                num_steps: total,
                num_complete_steps: complete,
                pr_url: pr_info.url,
                run_url: run.url,
            },
            other => todo!("unhandled status: {other}"),
        };

        tracing::debug!(pr = %pr_number, status = ?pr_result, "PR result");

        Ok(pr_result)
    }

    pub async fn fetch_workflows(
        &self,
        token: impl AsRef<str>,
        owner: impl AsRef<str>,
        repo: impl AsRef<str>,
    ) -> eyre::Result<Vec<WorkflowDetails>> {
        let token = token.as_ref();
        let owner = owner.as_ref();
        let repo = repo.as_ref();
        tracing::debug!(%owner, %repo, "fetching workflows");

        // requires `actions:read`
        match self
            .client
            .get(
                format!("/repos/{}/{}/actions/workflows", owner, repo),
                token,
                None::<()>,
            )
            .await
        {
            Ok(GetWorkflowsResponse { workflows }) => {
                tracing::debug!(?workflows, "got workflows for repo");
                Ok(workflows)
            }
            Err(e) => {
                tracing::warn!(error = %e, "error fetching workflows");
                eyre::bail!("error fetching workflows");
            }
        }
    }

    async fn fetch_pr_info(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        token: &str,
    ) -> eyre::Result<GetPullRequestResponse> {
        tracing::debug!("fetching pr info");
        // requires `contents:read`
        self.client
            .get(
                format!("/repos/{}/{}/pulls/{}", owner, repo, pr_number),
                token,
                None::<()>,
            )
            .await
            .wrap_err("fetching branch info")
    }

    async fn fetch_workflow_runs(
        &self,
        owner: &str,
        repo: &str,
        workflow_id: u64,
        branch_name: impl Into<String>,
        token: &str,
    ) -> eyre::Result<GetWorkflowRunsResponse> {
        tracing::debug!("fetching workflow runs");
        // requires `actions:read`
        self.client
            .get(
                format!(
                    "/repos/{}/{}/actions/workflows/{}/runs",
                    owner, repo, workflow_id,
                ),
                token,
                Some(GetWorkflowRunsQueryArgs {
                    branch: branch_name.into(),
                }),
            )
            .await
    }

    async fn fetch_run_jobs(
        &self,
        owner: &str,
        repo: &str,
        run_id: u64,
        token: &str,
    ) -> eyre::Result<GetRunJobsResponse> {
        // requires `actions:read`
        self.client
            .get(
                format!("/repos/{}/{}/actions/runs/{}/jobs", owner, repo, run_id,),
                token,
                None::<()>,
            )
            .await
    }
}

#[derive(Debug)]
struct ProgressResult {
    progress: f32,
    complete: u64,
    total: u64,
}

fn calculate_progress(jobs: &[RunJob]) -> ProgressResult {
    let mut n_steps_total = 0u64;
    let mut completed_steps = 0u64;
    for job in jobs {
        let n_steps = u64::try_from(job.steps.len()).unwrap();

        if job.status == "completed" {
            n_steps_total += n_steps;
            completed_steps += n_steps;
            continue;
        }

        for step in &job.steps {
            n_steps_total += 1;
            if step.status == "completed" {
                completed_steps += 1;
            }
        }
    }
    tracing::trace!(%completed_steps, %n_steps_total, "calculated progress percentage");

    // TODO: fallable cast

    ProgressResult {
        progress: (completed_steps as f32) / (n_steps_total as f32),
        complete: completed_steps,
        total: n_steps_total,
    }
}

#[derive(Debug, Serialize, Clone, Copy)]
pub enum Status {
    Queued,
    #[allow(dead_code)]
    InProgress(f32),
    Succeeded,
    Failed,
}

#[derive(Debug, Serialize, Clone)]
pub struct Pr {
    pub status: Status,
    pub title: String,
    pub description: String,
    pub num_steps: u64,
    pub num_complete_steps: u64,
    pub pr_url: String,
    pub run_url: String,
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use crate::{
        fetcher::{calculate_progress, ProgressResult},
        github::GetRunJobsResponse,
    };

    #[test]
    fn from_example() {
        let s = std::fs::read_to_string("testdata/in-progress-jobs.json").unwrap();
        let GetRunJobsResponse { jobs } = serde_json::from_str(&s).unwrap();
        let ProgressResult { progress, .. } = calculate_progress(&jobs);
        assert_abs_diff_eq!(progress, 0.6875, epsilon = 0.001);
    }
}
