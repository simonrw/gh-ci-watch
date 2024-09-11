#![allow(dead_code)]

use color_eyre::eyre::{self, Context};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, USER_AGENT},
    IntoUrl,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    // TODO: datetime
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct Head {
    #[serde(rename = "ref")]
    pub branch: String,
}
#[derive(Debug, Deserialize)]
pub struct GetPullRequestResponse {
    pub title: String,
    pub head: Head,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub display_title: String,
    pub run_attempt: u64,
    pub run_number: u64,
    // TODO: datetime
    pub run_started_at: String,
    pub head_commit: Commit,
}

#[derive(Debug, Deserialize)]
pub struct GetWorkflowRunsResponse {
    pub workflow_runs: Vec<WorkflowRun>,
}

#[derive(Serialize)]
pub struct GetWorkflowRunsQueryArgs {
    pub branch: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobStep {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunJob {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub steps: Vec<JobStep>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GetRunJobsResponse {
    pub jobs: Vec<RunJob>,
}

// Query client

pub struct GitHubClient {
    client: Client,
    token: String,
}

// Constructors
impl GitHubClient {
    pub fn from_env() -> eyre::Result<Self> {
        let token = std::env::var("GITHUB_TOKEN").wrap_err("no GitHub token found")?;

        let mut headers = HeaderMap::new();
        headers.append(USER_AGENT, HeaderValue::from_static("gh-ci-watch"));

        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .wrap_err("constructing HTTP client")?;
        Ok(Self { client, token })
    }

    pub fn get<T, Q>(&self, url: impl IntoUrl, query: Option<Q>) -> eyre::Result<T>
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
