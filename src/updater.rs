use std::io;
use std::io::Write;

use anyhow::{Context, Result, bail};
use colored::Colorize;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct RepoRelease {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize, Debug)]
struct Asset {
    name: String,
    download_url: String,
}

// ----------- CHECK VERSION -------------------

pub async fn show_display_msg() {
    let prefix = String::from('v');
    let current_version = format!("{}{}", prefix, env!("CARGO_PKG_VERSION"));
    let latest_version = latest_version().await;

    if latest_version == "ERROR" {
        // TODO: Add red color
        println!("ERROR: find the latest version");
        return;
    }

    if current_version == latest_version {
        start_updater(&latest_version);
    }
}
async fn check_repo() -> Result<RepoRelease, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/IgnacioToledoDev/ninja-linter/releases/latest")
        .header("User-Agent", "ninja-linter")
        .send()
        .await?
        .error_for_status()?;

    let release: RepoRelease = response.json().await?;

    Ok(release)
}

async fn latest_version() -> String {
    match check_repo().await {
        Ok(release) => release.tag_name,
        Err(_) => String::from("ERROR"),
    }
}

// ------------ UPDATER PROCESS ------------------

// TODO: refactor this
fn start_updater(latest: &str) {
    println!(
        "{}",
        format!(
            "New version available: {} → {}. can you update (y/o)",
            env!("CARGO_PKG_VERSION"),
            latest
        )
        .yellow()
    );

    let mut user_res = String::new();

    io::stdin()
        .read_line(&mut user_res)
        .expect("Failed to read line");

    let expected_res = String::from("y");

    if user_res.trim().to_lowercase() != expected_res {
        return;
    }

    println!("HELLO to download new version");

    // TODO: download to new version
}

fn platform_identifier() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-amd64",
        ("linux", "aarch64") => "linux-arm64",
        ("macos", "x86_64") => "darwin-amd64",
        ("macos", "aarch64") => "darwin-arm64",
        ("windows", "x86_64") => "windows-amd64",
        _ => panic!(
            "Plataforma no soportada: {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        ),
    }
}

fn find_asset_url(release: &RepoRelease) -> Result<String> {
    let platform = platform_identifier();

    release
        .assets
        .iter()
        .find(|a| a.name.contains(platform))
        .map(|a| a.download_url.clone())
        .with_context(|| format!("no se encontró un asset para la plataforma '{}'", platform))
}

fn download_to_temp(url: &str) -> Result<std::path::PathBuf> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "ninja-linter")
        .send()
        .context("fallo al descargar el asset")?;

    if !resp.status().is_success() {
        bail!("descarga falló con status {}", resp.status());
    }

    let bytes = resp
        .bytes()
        .context("fallo al leer el cuerpo de la descarga")?;

    let tmp_path = std::env::temp_dir().join(format!("{}-update", BIN_NAME));
    let mut file = std::fs::File::create(&tmp_path)?;
    file.write_all(&bytes)?;

    // En Unix, marcar como ejecutable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&tmp_path, perms)?;
    }

    Ok(tmp_path)
}
// --------------- test ----------------------

