use std::fs;
use std::io::Read;
use std::path::Path;

use base64::Engine;
use serde_json::Value;

use crate::Result;
use crate::data;

struct Repo {
    owner: String,
    name: String,
    description: String,
    language: Option<String>,
    stars: u64,
    forks: u64,
    issues: u64,
    contributors: u64,
    avatar: String,
}

pub fn run(out: &Path) -> Result<()> {
    fs::create_dir_all(out)?;

    for project in data::PROJECTS {
        let Some(repo) = project.repo else { continue };
        let file = project.image.trim_start_matches("/repos/");

        match share(repo).map(|(_, png)| png) {
            Ok(png) => {
                fs::write(out.join(file), png)?;
                println!("{repo:<28} -> {file}");
            }
            Err(error) => eprintln!("{repo}: {error}, keeping the current {file}"),
        }
    }

    Ok(())
}

fn get(url: &str) -> Result<ureq::Response> {
    let mut request = ureq::get(url)
        .timeout(std::time::Duration::from_secs(15))
        .set("User-Agent", "meetzli-portfolio")
        .set("Accept", "application/vnd.github+json");

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }

    Ok(request.call().map_err(|e| format!("{url}: {e}"))?)
}

fn json(url: &str) -> Result<Value> {
    Ok(serde_json::from_reader(get(url)?.into_reader())?)
}

fn fetch(repo: &str) -> Result<Repo> {
    let info = json(&format!("https://api.github.com/repos/{repo}"))?;
    let text = |key: &str| info[key].as_str().unwrap_or("").to_string();
    let count = |key: &str| info[key].as_u64().unwrap_or(0);

    let response = get(&format!(
        "https://api.github.com/repos/{repo}/contributors?per_page=1&anon=true"
    ))?;
    let last_page = response
        .header("Link")
        .and_then(|link| link.split(',').find(|part| part.contains("rel=\"last\"")))
        .and_then(|part| part.split("page=").last())
        .and_then(|rest| rest.split('>').next())
        .and_then(|page| page.parse::<u64>().ok());
    let contributors = match last_page {
        Some(pages) => pages,
        None => serde_json::from_reader::<_, Vec<Value>>(response.into_reader())
            .map(|v| v.len() as u64)
            .unwrap_or(0),
    };

    let avatar_url = info["owner"]["avatar_url"].as_str().ok_or("no avatar")?;
    let avatar = get(&format!("{avatar_url}&s=240"))?;
    let mime = avatar.content_type().to_string();
    let mut bytes = Vec::new();
    avatar.into_reader().read_to_end(&mut bytes)?;

    Ok(Repo {
        owner: info["owner"]["login"].as_str().unwrap_or("").to_string(),
        name: text("name"),
        description: text("description"),
        language: info["language"].as_str().map(str::to_string),
        stars: count("stargazers_count"),
        forks: count("forks_count"),
        issues: count("open_issues_count"),
        contributors,
        avatar: format!(
            "data:{mime};base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        ),
    })
}

pub fn valid(repo: &str) -> bool {
    github_share::route(&format!("/gh/{repo}")).is_some_and(|(_, image)| !image)
}

pub fn share(repo: &str) -> Result<(String, Vec<u8>)> {
    if !valid(repo) {
        return Err("invalid repository".into());
    }
    let repo = fetch(repo)?;
    let info = github_share::Repo {
        owner: github_share::Owner {
            login: repo.owner,
            avatar_url: String::new(),
        },
        name: repo.name,
        description: Some(repo.description.clone()),
        language: repo.language,
        stargazers_count: repo.stars,
        forks_count: repo.forks,
        open_issues_count: repo.issues,
        private: false,
        contributors: repo.contributors,
        avatar: repo.avatar,
    };
    Ok((repo.description, github_share::png(&info)?))
}

pub fn html(repo: &str, description: &str, base: &str) -> String {
    github_share::html(repo, description, base)
}
