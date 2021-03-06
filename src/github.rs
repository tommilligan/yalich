use anyhow::{Context, Result};
use log::debug;
use reqwest::blocking::Client;
use serde_derive::Deserialize;
use url::Url;

use crate::core::Dependency;

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

    pub fn repo(&self, organisation: &str, repo: &str) -> Result<Repo> {
        // Validate and extract API url from homepage
        let url = Url::parse(&format!(
            "https://api.github.com/repos/{}/{}",
            organisation, repo,
        ))
        .with_context(|| format!("Invalid URL for Github API '{}/{}'.", organisation, repo))?;

        self.client
            .get(url)
            .send()
            .with_context(|| format!("Github request for '{}/{}' failed.", organisation, repo))?
            .json()
            .with_context(|| {
                format!(
                    "JSON deserialization for '{}/{}' failed.",
                    organisation, repo
                )
            })
    }
}

/// Validate and extract repo from homepage url.
fn homepage_to_repo(mut homepage: &str) -> Option<(&str, &str)> {
    homepage = homepage
        .split('#')
        .next()
        .expect("Split always results in at least one part.");

    if homepage.ends_with('/') {
        homepage = &homepage[..homepage.len() - 1]
    };

    if !homepage.contains("github.com") {
        return None;
    };

    let html_url_parts: Vec<_> = homepage.rsplitn(3, '/').collect();
    if let Some(repo) = html_url_parts.get(0) {
        if let Some(organisation) = html_url_parts.get(1) {
            return Some((organisation, repo));
        }
    }

    None
}

pub struct Enricher<'a> {
    github: &'a Github<'a>,
}

impl<'a> Enricher<'a> {
    pub fn new(github: &'a Github) -> Self {
        Self { github }
    }

    pub fn enrich(&self, mut dependency: Dependency) -> Result<Dependency> {
        // If language doesn't have license
        if dependency.license.is_none() {
            // And we have a homepage
            if let Some(homepage) = &dependency.homepage {
                // And it's Github
                if let Some((organisation, repo)) = homepage_to_repo(&homepage) {
                    debug!("Falling back to Github for {}/{}", organisation, repo);
                    let repo = self.github.repo(organisation, repo)?;
                    if let Some(repo_license) = repo.license {
                        if repo_license.name != "Other" {
                            dependency.license = Some(repo_license.name)
                        }
                    }
                }
            }
        };
        Ok(dependency)
    }
}
