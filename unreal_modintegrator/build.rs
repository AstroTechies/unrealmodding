use std::env;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use reqwest::{
    blocking,
    header::{self, HeaderMap},
};
use serde::Deserialize;

const ASSET_REPO: &str = "AstroTechies/ModIntegrator";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(feature = "no_bulk_data")]
    compile_error!("no_bulk_data feature is not supported yet.");

    #[cfg(not(any(feature = "ue4_23", feature = "ue4_24", feature = "ue4_25")))]
    compile_error!("No UE version feature enabled.");

    let out_dir = PathBuf::from(&env::var_os("OUT_DIR").expect("Failed to read OUT_DIR"));

    let project_dir = out_dir.join("ModIntegrator");
    fs::remove_dir_all(&project_dir).unwrap_or(());

    let use_prebuilt = env::var_os("USE_PREBUILT_ASSETS").is_some();

    if use_prebuilt {
        download_release(&out_dir);
    } else {
        cook_now(&out_dir);
    }
}

fn download_release(out_dir: &Path) {
    // download the files from the latest release on github

    let project_dir = out_dir.join("ModIntegrator");

    let mut headers = HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        "reqwest/unreal_modintegrator-buildscript"
            .parse()
            .expect("Invalid user agent"),
    );

    let api_response = blocking::Client::new()
        .get(format!(
            "https://api.github.com/repos/{}/releases/latest",
            ASSET_REPO
        ))
        .headers(headers.clone())
        .send()
        .unwrap();

    #[derive(Debug, Deserialize)]
    struct Release {
        assets: Vec<ReleaseAsset>,
    }
    #[derive(Debug, Deserialize)]
    struct ReleaseAsset {
        name: String,
        browser_download_url: String,
    }

    let release: Release = api_response.json().unwrap();

    #[cfg(all(feature = "ue4_23", not(feature = "no_bulk_data")))]
    let file_name = "ue4_23.zip";

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == file_name)
        .expect("Could not find correct file in release.");

    fs::create_dir(&project_dir).expect("Failed to create Integrator dir");
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(project_dir.join(file_name))
        .expect("Could not open file");

    let mut response = blocking::Client::new()
        .get(&asset.browser_download_url)
        .headers(headers)
        .send()
        .unwrap();

    io::copy(&mut response, &mut file).expect("Could not copy downloaded file");

    let integrator_dir =
        project_dir.join("Saved/Cooked/WindowsNoEditor/ModIntegrator/Content/Integrator");
    fs::create_dir_all(&integrator_dir).expect("Failed to create Integrator subdirs");

    zip_extract::extract(&mut file, &integrator_dir, false).expect("Failed to extract zip file");
}

fn cook_now(out_dir: &PathBuf) {
    let mut git = Command::new("git")
        .args([
            "clone",
            format!("https://github.com/{}.git", ASSET_REPO).as_str(),
        ])
        .current_dir(out_dir)
        .spawn()
        .expect("failed to clone repo");
    let status = git.wait().expect("failed to run git");
    if !status.success() {
        panic!("git failed to finish {}", status);
    }

    let version_selector = env::var_os("UE_VERSION_SELECTOR").expect("UE_VERSION_SELECTOR not set");
    let engine_path = env::var_os("UE_PATH").expect("UE_PATH not set");

    let project_file = out_dir.join("ModIntegrator").join("ModIntegrator.uproject");
    let mut version_selector = Command::new(version_selector)
        .arg("/switchversionsilent")
        .arg(project_file.as_os_str())
        .arg(&engine_path)
        .spawn()
        .expect("failed to launch UnrealVersionSelector");
    let status = version_selector
        .wait()
        .expect("failed to launch UnrealVersionSelector");
    if !status.success() {
        panic!("UnrealVersionSelector failed to finish {}", status);
    }

    let ue4_cmd_path =
        Path::new(&engine_path).join("Engine\\\\Binaries\\\\Win64\\\\UE4Editor-Cmd.exe");

    let mut cook = Command::new(ue4_cmd_path)
        .arg(project_file.as_os_str())
        .args([
            "-run=cook",
            "-targetplatform=WindowsNoEditor",
            "-CrashForUAT",
            "-unattended",
        ])
        .spawn()
        .expect("failed to launch UE4Editor-Cmd");
    let status = cook.wait().expect("failed to launch UE4Editor-Cmd");
    if !status.success() {
        panic!("UE4Editor-Cmd failed to finish {}", status);
    }
}
