use crate::error::{GitHubError, Result};
use desktop_core::models::{GitHubUser, Issue, PullRequest};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GitHubClient {
    token: Option<String>,
    base_url: String,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct ApiUser {
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiUserRef {
    login: String,
}

#[derive(Debug, Deserialize)]
struct ApiGitRef {
    #[serde(rename = "ref")]
    branch_ref: String,
}

#[derive(Debug, Deserialize)]
struct ApiPullRequest {
    number: u64,
    title: String,
    body: Option<String>,
    state: String,
    user: ApiUserRef,
    head: ApiGitRef,
    base: ApiGitRef,
    html_url: String,
    #[serde(default)]
    draft: bool,
}

#[derive(Debug, Deserialize)]
struct ApiIssue {
    number: u64,
    title: String,
    state: String,
    user: ApiUserRef,
    html_url: String,
    // Pull requests also appear in issues endpoint, distinguished by pull_request key
    pull_request: Option<serde_json::Value>,
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::from_env()
    }
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("desktop-rs/0.1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            token,
            base_url: "https://api.github.com".to_string(),
            http,
        }
    }

    /// Automatically discovers a GitHub token from environment variables
    /// (`GITHUB_TOKEN`, `GH_TOKEN`) or the `gh` CLI credentials.
    pub fn from_env() -> Self {
        let token = std::env::var("GITHUB_TOKEN")
            .ok()
            .or_else(|| std::env::var("GH_TOKEN").ok())
            .or_else(Self::token_from_gh_cli);

        Self::new(token)
    }

    fn token_from_gh_cli() -> Option<String> {
        let output = Command::new("gh").args(["auth", "token"]).output().ok()?;
        if output.status.success() {
            let t = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !t.is_empty() {
                return Some(t);
            }
        }
        None
    }

    pub fn has_auth(&self) -> bool {
        self.token.is_some()
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }

    fn request_builder(&self, method: reqwest::Method, endpoint: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut builder = self
            .http
            .request(method, &url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");

        if let Some(token) = &self.token {
            builder = builder.header("Authorization", format!("Bearer {token}"));
        }

        builder
    }

    /// Parses owner and repository name from a git remote URL
    pub fn parse_owner_repo(remote_url: &str) -> Option<(String, String)> {
        let trimmed = remote_url.trim().trim_end_matches(".git");

        // Format: git@github.com:owner/repo
        if let Some(colon_idx) = trimmed.find(':') {
            if trimmed.contains("github.com") && !trimmed.starts_with("http") {
                let path = &trimmed[colon_idx + 1..];
                let parts: Vec<&str> = path.split('/').collect();
                if parts.len() == 2 {
                    return Some((parts[0].to_string(), parts[1].to_string()));
                }
            }
        }

        // Format: https://github.com/owner/repo
        if let Some(github_idx) = trimmed.find("github.com/") {
            let path = &trimmed[github_idx + 11..];
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                return Some((parts[0].to_string(), parts[1].to_string()));
            }
        }

        None
    }

    /// Fetches the authenticated GitHub user profile
    pub async fn get_current_user(&self) -> Result<GitHubUser> {
        if self.token.is_none() {
            return Err(GitHubError::MissingToken);
        }

        let resp = self
            .request_builder(reqwest::Method::GET, "/user")
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(GitHubError::ApiError { status, message });
        }

        let user: ApiUser = resp.json().await?;
        Ok(GitHubUser {
            login: user.login,
            name: user.name,
            email: user.email,
            avatar_url: user.avatar_url,
        })
    }

    /// Lists open or specified state pull requests for a repository
    pub async fn list_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> Result<Vec<PullRequest>> {
        let state_param = state.unwrap_or("open");
        let endpoint = format!("/repos/{owner}/{repo}/pulls?state={state_param}&per_page=30");

        let resp = self
            .request_builder(reqwest::Method::GET, &endpoint)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(GitHubError::ApiError { status, message });
        }

        let prs: Vec<ApiPullRequest> = resp.json().await?;
        Ok(prs
            .into_iter()
            .map(|p| PullRequest {
                number: p.number,
                title: p.title,
                body: p.body,
                state: p.state,
                author: p.user.login,
                head_branch: p.head.branch_ref,
                base_branch: p.base.branch_ref,
                html_url: p.html_url,
                is_draft: p.draft,
            })
            .collect())
    }

    /// Gets details of a specific pull request
    pub async fn get_pull_request(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<PullRequest> {
        let endpoint = format!("/repos/{owner}/{repo}/pulls/{number}");

        let resp = self
            .request_builder(reqwest::Method::GET, &endpoint)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(GitHubError::ApiError { status, message });
        }

        let p: ApiPullRequest = resp.json().await?;
        Ok(PullRequest {
            number: p.number,
            title: p.title,
            body: p.body,
            state: p.state,
            author: p.user.login,
            head_branch: p.head.branch_ref,
            base_branch: p.base.branch_ref,
            html_url: p.html_url,
            is_draft: p.draft,
        })
    }

    /// Lists open issues for referencing in commit messages
    pub async fn list_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>> {
        let endpoint = format!("/repos/{owner}/{repo}/issues?state=open&per_page=30");

        let resp = self
            .request_builder(reqwest::Method::GET, &endpoint)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(GitHubError::ApiError { status, message });
        }

        let items: Vec<ApiIssue> = resp.json().await?;
        // Filter out pull requests which are returned by the issues endpoint
        let issues = items
            .into_iter()
            .filter(|i| i.pull_request.is_none())
            .map(|i| Issue {
                number: i.number,
                title: i.title,
                state: i.state,
                author: i.user.login,
                html_url: i.html_url,
            })
            .collect();

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_owner_repo_https() {
        let url = "https://github.com/desktop/desktop.git";
        let res = GitHubClient::parse_owner_repo(url);
        assert_eq!(res, Some(("desktop".to_string(), "desktop".to_string())));
    }

    #[test]
    fn test_parse_owner_repo_ssh() {
        let url = "git@github.com:bhubbard/desktop-rs.git";
        let res = GitHubClient::parse_owner_repo(url);
        assert_eq!(
            res,
            Some(("bhubbard".to_string(), "desktop-rs".to_string()))
        );
    }
}
