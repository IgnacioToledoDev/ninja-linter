use std::io;
use std::io::Write;

use anyhow::{Context, Result, bail};
// use colored::Colorize;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct RepoRelease {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize, Debug)]
struct Asset {
    name: String,
    browser_download_url: String,
}

// ----------- CHECK VERSION -------------------

pub async fn show_display_msg() {
    let prefix = String::from('v');
    let current_version = format!("{}{}", prefix, env!("CARGO_PKG_VERSION"));
    let latest_version = latest_version().await;

    if latest_version.tag_name == "ERROR" {
        // TODO: Add red color
        println!("ERROR: find the latest version");
        return;
    }

    if is_newer(&latest_version.tag_name, &current_version) {
        start_updater(&latest_version).await;
    }
}

fn parse_semver(v: &str) -> Option<(u64, u64, u64)> {
    let s = v.trim_start_matches('v');
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

fn is_newer(candidate: &str, baseline: &str) -> bool {
    match (parse_semver(candidate), parse_semver(baseline)) {
        (Some(c), Some(b)) => c > b,
        _ => candidate > baseline,
    }
}
async fn fetch_release(url: &str) -> Result<RepoRelease> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "ninja-linter")
        .send()
        .await
        .context("fallo al contactar la API de GitHub")?;

    if !response.status().is_success() {
        bail!("GitHub respondió con status {}", response.status());
    }

    response
        .json::<RepoRelease>()
        .await
        .context("fallo al parsear el JSON de la release")
}

async fn check_repo() -> Result<RepoRelease> {
    fetch_release("https://api.github.com/repos/IgnacioToledoDev/ninja-linter/releases/latest")
        .await
}

async fn latest_version() -> RepoRelease {
    match check_repo().await {
        Ok(release) => release,
        // TODO: Better handler errors this
        Err(_) => RepoRelease {
            tag_name: String::from("ERROR"),
            assets: vec![],
        },
    }
}

// ------------ UPDATER PROCESS ------------------
// TODO: implementation of aarch64-unknown-linux-gnu pending
fn platform_identifier() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
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
        .map(|a| a.browser_download_url.clone())
        .with_context(|| format!("no se encontró un asset para la plataforma '{}'", platform))
}

async fn download_to_temp(url: &str) -> Result<std::path::PathBuf> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "ninja-linter")
        .send()
        .await
        .context("fallo al descargar el asset")?;

    if !resp.status().is_success() {
        bail!("descarga falló con status {}", resp.status());
    }

    let bytes = resp
        .bytes()
        .await
        .context("fallo al leer el cuerpo de la descarga")?;

    let tmp_path = std::env::temp_dir().join(format!("{}-update", "ninja-linter"));
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

// TODO: refactor this
async fn start_updater(latest: &RepoRelease) {
    println!(
        "New version available: {} → {}. can you update (y/o)",
        env!("CARGO_PKG_VERSION"),
        latest.tag_name
    );

    let user_res = tokio::task::block_in_place(|| {
        let mut buf = String::new();
        io::stdin()
            .read_line(&mut buf)
            .expect("Failed to read line");
        buf
    });

    if user_res.trim().to_lowercase() != "y" {
        return;
    }

    let asset_url = match find_asset_url(latest) {
        Ok(url) => url,
        Err(e) => {
            println!("Cannot find asset: {}", e);
            return;
        }
    };

    let path = match download_to_temp(&asset_url).await {
        Ok(p) => p,
        Err(e) => {
            println!("Cannot download update: {}", e);
            return;
        }
    };

    if let Err(e) = self_replace::self_replace(&path) {
        println!("Cannot replace binary: {}", e);
        return;
    }

    let _ = std::fs::remove_file(&path);

    println!(
        "Ready: Ninja Linter updated to version: {}",
        &latest.tag_name
    );
}

// --------------- test ----------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_semver ---

    #[test]
    fn test_parse_semver_with_v_prefix() {
        assert_eq!(parse_semver("v0.8.0"), Some((0, 8, 0)));
    }

    #[test]
    fn test_parse_semver_without_prefix() {
        assert_eq!(parse_semver("1.10.3"), Some((1, 10, 3)));
    }

    #[test]
    fn test_parse_semver_invalid() {
        assert_eq!(parse_semver("ERROR"), None);
        assert_eq!(parse_semver("v1.2"), None);
    }

    // --- is_newer ---

    #[test]
    fn test_is_newer_detects_update() {
        assert!(is_newer("v0.9.0", "v0.8.0"));
    }

    #[test]
    fn test_is_newer_same_version() {
        assert!(!is_newer("v0.8.0", "v0.8.0"));
    }

    #[test]
    fn test_is_newer_older_does_not_trigger() {
        assert!(!is_newer("v0.7.0", "v0.8.0"));
    }

    #[test]
    fn test_is_newer_double_digit_minor() {
        // string comparison would fail here: "v0.10.0" < "v0.9.0" lexicographically
        assert!(is_newer("v0.10.0", "v0.9.0"));
    }

    #[test]
    fn test_is_newer_major_bump() {
        assert!(is_newer("v1.0.0", "v0.99.99"));
    }

    // --- platform_identifier ---

    #[test]
    fn test_platform_identifier_returns_known_value() {
        let platform = platform_identifier();
        let known = [
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
            "x86_64-pc-windows-msvc",
        ];
        assert!(
            known.contains(&platform),
            "unexpected platform: {}",
            platform
        );
    }

    // --- find_asset_url ---

    #[test]
    fn test_find_asset_url_found() {
        let platform = platform_identifier();
        let release = RepoRelease {
            tag_name: String::from("v1.0.0"),
            assets: vec![Asset {
                name: format!("ninja-linter-{}.tar.gz", platform),
                browser_download_url: String::from("https://example.com/asset"),
            }],
        };
        let url = find_asset_url(&release).unwrap();
        assert_eq!(url, "https://example.com/asset");
    }

    #[test]
    fn test_find_asset_url_not_found() {
        let release = RepoRelease {
            tag_name: String::from("v1.0.0"),
            assets: vec![Asset {
                name: String::from("ninja-linter-unknown-platform.tar.gz"),
                browser_download_url: String::from("https://example.com/asset"),
            }],
        };
        assert!(find_asset_url(&release).is_err());
    }

    #[test]
    fn test_find_asset_url_empty_assets() {
        let release = RepoRelease {
            tag_name: String::from("v1.0.0"),
            assets: vec![],
        };
        assert!(find_asset_url(&release).is_err());
    }

    #[test]
    fn test_find_asset_url_picks_correct_from_multiple() {
        let platform = platform_identifier();
        let release = RepoRelease {
            tag_name: String::from("v1.0.0"),
            assets: vec![
                Asset {
                    name: String::from("ninja-linter-other-platform.tar.gz"),
                    browser_download_url: String::from("https://example.com/wrong"),
                },
                Asset {
                    name: format!("ninja-linter-{}.tar.gz", platform),
                    browser_download_url: String::from("https://example.com/correct"),
                },
            ],
        };
        let url = find_asset_url(&release).unwrap();
        assert_eq!(url, "https://example.com/correct");
    }

    // --- JSON deserialization ---

    #[test]
    fn test_repo_release_deserializes() {
        let json = r#"{
            "tag_name": "v1.2.3",
            "assets": [
                {
                    "name": "ninja-linter-linux-amd64",
                    "browser_download_url": "https://github.com/example/releases/download/v1.2.3/ninja-linter-linux-amd64"
                }
            ]
        }"#;
        let release: RepoRelease = serde_json::from_str(json).unwrap();
        assert_eq!(release.tag_name, "v1.2.3");
        assert_eq!(release.assets.len(), 1);
        assert_eq!(release.assets[0].name, "ninja-linter-linux-amd64");
        assert_eq!(
            release.assets[0].browser_download_url,
            "https://github.com/example/releases/download/v1.2.3/ninja-linter-linux-amd64"
        );
    }

    #[test]
    fn test_repo_release_deserializes_empty_assets() {
        let json = r#"{ "tag_name": "v1.0.0", "assets": [] }"#;
        let release: RepoRelease = serde_json::from_str(json).unwrap();
        assert_eq!(release.tag_name, "v1.0.0");
        assert!(release.assets.is_empty());
    }

    #[test]
    fn test_repo_release_invalid_json_errors() {
        let result = serde_json::from_str::<RepoRelease>("{ invalid }");
        assert!(result.is_err());
    }

    #[test]
    fn test_repo_release_missing_field_errors() {
        // assets field missing
        let json = r#"{ "tag_name": "v1.0.0" }"#;
        let result = serde_json::from_str::<RepoRelease>(json);
        assert!(result.is_err());
    }

    // --- fetch_release (HTTP mock) ---

    #[tokio::test]
    async fn test_fetch_release_success() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/releases/latest")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"tag_name":"v2.0.0","assets":[{"name":"ninja-linter-linux-amd64","browser_download_url":"https://example.com/bin"}]}"#,
            )
            .create_async()
            .await;

        let url = format!("{}/releases/latest", server.url());
        let release = fetch_release(&url).await.unwrap();
        assert_eq!(release.tag_name, "v2.0.0");
        assert_eq!(release.assets.len(), 1);
        assert_eq!(release.assets[0].name, "ninja-linter-linux-amd64");
    }

    #[tokio::test]
    async fn test_fetch_release_http_error_returns_err() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/releases/latest")
            .with_status(404)
            .create_async()
            .await;

        let url = format!("{}/releases/latest", server.url());
        assert!(fetch_release(&url).await.is_err());
    }

    #[tokio::test]
    async fn test_fetch_release_server_error_returns_err() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/releases/latest")
            .with_status(500)
            .create_async()
            .await;

        let url = format!("{}/releases/latest", server.url());
        assert!(fetch_release(&url).await.is_err());
    }

    #[tokio::test]
    async fn test_fetch_release_invalid_json_returns_err() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/releases/latest")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("not json at all")
            .create_async()
            .await;

        let url = format!("{}/releases/latest", server.url());
        assert!(fetch_release(&url).await.is_err());
    }

    // --- download_to_temp (HTTP mock) ---

    #[tokio::test]
    async fn test_download_to_temp_success() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/asset")
            .with_status(200)
            .with_body(b"fake-binary-content".as_ref())
            .create_async()
            .await;

        let url = format!("{}/asset", server.url());
        let path = download_to_temp(&url).await.unwrap();
        assert!(path.exists());
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_download_to_temp_http_error_returns_err() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/asset")
            .with_status(403)
            .create_async()
            .await;

        let url = format!("{}/asset", server.url());
        assert!(download_to_temp(&url).await.is_err());
    }
}
