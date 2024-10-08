use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    path::PathBuf,
};

use cmake::Config;
use git2::{Oid, Repository};

const ASSET_REPO: &str = "AstroTechies/UnrealCppLoader";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=USE_PRECOMPILED_CPP_LOADER");

    let use_prebuilt = env::var_os("USE_PRECOMPILED_CPP_LOADER")
        .map(|e| e == "1")
        .unwrap_or(false);

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let mod_loader_dir = out_dir.join("UnrealCppLoader");
    let _ = fs::remove_dir_all(&mod_loader_dir);

    let (unreal_mod_loader_path, xinput_proxy_path) = match use_prebuilt {
        true => download(&mod_loader_dir)?,
        false => compile(&mod_loader_dir)?,
    };

    println!(
        "cargo:rustc-env=CPP_LOADER_DLL_PATH={}",
        unreal_mod_loader_path.to_str().unwrap()
    );

    println!(
        "cargo:rustc-env=CPP_LOADER_PROXY_PATH={}",
        xinput_proxy_path.to_str().unwrap()
    );

    Ok(())
}

fn download(mod_loader_dir: &PathBuf) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    fs::create_dir_all(mod_loader_dir)?;

    let loader_file_path = mod_loader_dir.join("UnrealCppLoader.dll");
    let proxy_file_path = mod_loader_dir.join("xinput1_3.dll");

    let latest_release = github_helpers::get_latest_release(ASSET_REPO)?;

    #[cfg(debug_assertions)]
    let debug = true;
    #[cfg(not(debug_assertions))]
    let debug = false;

    #[cfg(feature = "gui")]
    let enable_gui = true;
    #[cfg(not(feature = "gui"))]
    let enable_gui = false;

    let expected_loader_name = format!(
        "UnrealCppLoader-{}-{}.dll",
        match enable_gui {
            true => "gui",
            false => "nogui",
        },
        match debug {
            true => "debug",
            false => "release",
        }
    );

    let loader_asset = latest_release
        .assets
        .iter()
        .find(|e| e.name == expected_loader_name)
        .unwrap();
    let proxy_asset = latest_release
        .assets
        .iter()
        .find(|e| e.name == "xinput1_3.dll")
        .unwrap();

    let mut loader_file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(true)
        .open(&loader_file_path)?;
    loader_asset.download(&mut loader_file)?;

    let mut proxy_file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(true)
        .open(&proxy_file_path)?;
    proxy_asset.download(&mut proxy_file)?;

    Ok((loader_file_path, proxy_file_path))
}

fn compile(mod_loader_dir: &PathBuf) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    let repository = Repository::clone(
        &format!("https://github.com/{ASSET_REPO}.git"),
        mod_loader_dir,
    )?;

    let oid_str = "b186f54ffd079a1023cdbddb96e07c7ddd1676af";
    let oid = Oid::from_str(oid_str)?;
    let commit = repository.find_commit(oid)?;

    repository.branch(oid_str, &commit, false)?;

    let mut build_config = Config::new(mod_loader_dir.to_str().unwrap());

    build_config
        .configure_arg("-DFETCHCONTENT_QUIET=OFF")
        .configure_arg(format!(
            "-DUNREALCPPLOADER_VERSION={}",
            env!("CARGO_PKG_VERSION")
        ));

    #[cfg(debug_assertions)]
    {
        build_config.configure_arg("-DCMAKE_BUILD_TYPE=Debug");
    }
    #[cfg(not(debug_assertions))]
    {
        build_config.configure_arg("-DCMAKE_BUILD_TYPE=Release");
    }

    #[cfg(feature = "gui")]
    {
        build_config.configure_arg("-DENABLE_GUI=ON");
    }

    let built = build_config.build();

    let build_path = built.join("build");

    #[cfg(debug_assertions)]
    let debug = true;
    #[cfg(not(debug_assertions))]
    let debug = false;

    let unreal_mod_loader_path = build_path.join("UnrealCppLoader");
    let unreal_mod_loader_path = match debug {
        true => unreal_mod_loader_path.join("Debug"),
        false => unreal_mod_loader_path.join("Release"),
    }
    .join("UnrealCppLoader.dll");

    let xinput_proxy_path = build_path.join("xinput1_3");
    let xinput_proxy_path = match debug {
        true => xinput_proxy_path.join("Debug"),
        false => xinput_proxy_path.join("Release"),
    }
    .join("xinput1_3.dll");

    Ok((unreal_mod_loader_path, xinput_proxy_path))
}
