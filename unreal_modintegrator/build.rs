use std::{env, fs, path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let project_dir = Path::new(&out_dir).join("ModIntegrator");
    fs::remove_dir_all(&project_dir);

    let mut git = Command::new("git")
        .args(["clone", "git@github.com:AstroTechies/ModIntegrator.git"])
        .current_dir(&out_dir)
        .spawn()
        .expect("failed to clone repo");
    let status = git.wait().expect("failed to run git");
    if !status.success() {
        panic!("git failed to finish {}", status);
    }

    let version_selector = env::var_os("UE_VERSION_SELECTOR").expect("UE_VERISON_SELECTOR not set");
    let engine_path = env::var_os("UE_PATH").expect("UE_PATH not set");

    let project_file = project_dir.join("ModIntegrator.uproject");
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
