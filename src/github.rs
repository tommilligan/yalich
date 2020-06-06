use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde_derive::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct License {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Repo {
    pub full_name: String,
    pub license: Option<License>,
    pub html_url: String,
}

pub struct Github<'a> {
    client: &'a Client,
}

impl<'a> Github<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub fn fetch_repo_from_homepage(&self, html_url: &str) -> Result<Repo> {
        // Validate and extract API url from homepage
        if !html_url.contains("github.com") {
            return Err(anyhow!("Homepage {} was not for Github", html_url));
        };
        let html_url_parts: Vec<_> = html_url.rsplitn(3, '/').collect();
        let organisation = html_url_parts
            .get(1)
            .with_context(|| format!("Homepage {} did not have an organisation.", html_url))?;
        let repo = html_url_parts
            .get(0)
            .with_context(|| format!("Homepage {} did not have a repo name.", html_url))?;
        let url = Url::parse(&format!(
            "https://api.github.com/repos/{}/{}",
            organisation, repo,
        ))
        .with_context(|| format!("Invalid URL for Github API '{}'.", html_url))?;

        self.client
            .get(url)
            .send()
            .with_context(|| format!("Github request for '{}' failed.", html_url))?
            .json()
            .with_context(|| format!("JSON deserialization for '{}' failed.", html_url))
    }
}
