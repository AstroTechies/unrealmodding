use std::error::Error;
use std::fs::File;
use std::io;

use reqwest::{
    blocking,
    header::{self, HeaderMap},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Release {
    pub assets: Vec<ReleaseAsset>,
}
#[derive(Debug, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

impl ReleaseAsset {
    pub fn download(&self, file: &mut File) -> Result<(), Box<dyn Error>> {
        let mut response = blocking::Client::new()
            .get(&self.browser_download_url)
            .headers(get_headers())
            .send()?;

        io::copy(&mut response, file)?;

        Ok(())
    }
}

fn get_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        "reqwest/unrealmodding-github_helpers"
            .parse()
            .expect("Invalid user agent"),
    );

    headers
}

pub fn get_latest_release(repo_url: &str) -> Result<Release, Box<dyn Error>> {
    let headers = get_headers();
    let api_response = blocking::Client::new()
        .get(format!(
            "https://api.github.com/repos/{repo_url}/releases/latest"
        ))
        .headers(headers)
        .send()?;

    let release = api_response.json()?;

    Ok(release)
}
