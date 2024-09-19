#![allow(dead_code)]

use color_eyre::eyre::{self, Context};
use reqwest::{
    header::{HeaderMap, HeaderValue, USER_AGENT},
    Client, IntoUrl,
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
    #[serde(rename = "body")]
    pub description: String,
    #[serde(rename = "html_url")]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDetails {
    pub id: u64,
    pub name: String,
    #[serde(rename = "path")]
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetWorkflowsResponse {
    pub workflows: Vec<WorkflowDetails>,
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

#[derive(Clone)]
pub struct GitHubClient {
    client: Client,
    // TODO: borrow
    base_url: String,
}

// Constructors
impl GitHubClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let mut headers = HeaderMap::new();
        headers.append(USER_AGENT, HeaderValue::from_static("gh-ci-watch"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("programming error");
        Self {
            client,
            base_url: base_url.into(),
        }
    }

    pub async fn get<T, Q>(
        &self,
        path: impl Into<String>,
        token: &str,
        query: Option<Q>,
    ) -> eyre::Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
        Q: Serialize,
    {
        let path = path.into();
        let url = format!("{}{}", &self.base_url, &path);
        let mut builder = self.client.get(url).bearer_auth(token);
        if let Some(query) = &query {
            builder = builder.query(query);
        }

        tracing::debug!("sending http request");
        let response = builder.send().await.wrap_err("sending GET request")?;
        if let Err(e) = response.error_for_status_ref() {
            tracing::warn!(error = %e, "bad status from GitHub");
            eyre::bail!("bad error status: {e}");
        }
        tracing::debug!("got http response");
        response.json().await.wrap_err("decoding JSON response")
    }
}
