use crate::github::{
    GetPullRequestResponse, GetRunJobsResponse, GetWorkflowRunsQueryArgs, GetWorkflowRunsResponse,
    GitHubClient, RunJob,
};
use color_eyre::eyre::{self, Context};
use serde::Serialize;

// The workflow id of the tests-ext.yml tests
// TODO: how to calculate progress? Is the list of jobs/steps consistent?
const EXT_TESTS_NUMBER: i64 = 107927392;

pub struct Fetcher {
    client: GitHubClient,
}

impl Fetcher {
    pub fn new() -> Self {
        let client = GitHubClient::from_env().unwrap();
        Self { client }
    }

    pub async fn fetch(
        &self,
        owner: impl AsRef<str>,
        repo: impl AsRef<str>,
        pr_number: u64,
    ) -> eyre::Result<Pr> {
        let owner = owner.as_ref();
        let repo = repo.as_ref();
        tracing::debug!("fetching pr info");
        let pr_info: GetPullRequestResponse = self
            .client
            .get(
                format!(
                    "https://api.github.com/repos/{}/{}/pulls/{}",
                    owner, repo, pr_number,
                ),
                None::<()>,
            )
            .await
            .wrap_err("fetching branch info")?;

        // fetch workflow runs for branch
        tracing::debug!("fetching workflow runs");
        let GetWorkflowRunsResponse { mut workflow_runs } = self
            .client
            .get(
                format!(
                    "https://api.github.com/repos/{}/{}/actions/workflows/{}/runs",
                    owner, repo, EXT_TESTS_NUMBER
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
        let pr_result = match run.status.as_str() {
            "completed" => match run.conclusion.as_deref() {
                Some("failure") => {
                    Pr {
                        status: Status::Failed,
                    }
                    // tracing::debug!(before = ?pr.status, after = ?Status::Failed, "updating status");
                    // pr.status = Status::Failed;
                }
                Some("success") => Pr {
                    status: Status::Succeeded,
                },
                other => {
                    todo!("unhandled combination of status: completed and conclusion: {other:?}")
                }
            },
            "queued" => Pr {
                status: Status::Queued,
            },
            "in_progress" => {
                // get run jobs
                tracing::debug!("fetching jobs for run");
                let GetRunJobsResponse { jobs } = self
                    .client
                    .get(
                        format!(
                            "https://api.github.com/repos/{}/{}/actions/runs/{}/jobs",
                            owner, repo, run.id
                        ),
                        None::<()>,
                    )
                    .await
                    .wrap_err("fetching run jobs")?;

                let progress = calculate_progress(&jobs).unwrap_or(0.0);
                let status = Status::InProgress(progress);
                Pr { status }
            }
            "pending" => Pr {
                status: Status::Queued,
            },
            other => todo!("unhandled status: {other}"),
        };

        tracing::debug!(pr = %pr_number, status = ?pr_result, "PR result");

        Ok(pr_result)
    }
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
    tracing::trace!(%completed_steps, %n_steps_total, "calculated progress percentage");
    Ok(completed_steps / (n_steps_total as f32))
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
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use crate::{fetcher::calculate_progress, github::GetRunJobsResponse};

    #[test]
    fn from_example() {
        let s = std::fs::read_to_string("testdata/in-progress-jobs.json").unwrap();
        let GetRunJobsResponse { jobs } = serde_json::from_str(&s).unwrap();
        let progress = calculate_progress(&jobs).unwrap();
        assert_abs_diff_eq!(progress, 0.6875, epsilon = 0.001);
    }
}
