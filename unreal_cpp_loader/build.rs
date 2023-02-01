use std::{env, error::Error, fs, path::PathBuf};

use cmake::Config;
use git2::{Oid, Repository};

fn main() -> Result<(), Box<dyn Error>> {
    // todo pre-release: allow binary releases downloads instead
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let mod_loader_dir = out_dir.join("UnrealModLoader");
    let _ = fs::remove_dir_all(&mod_loader_dir);

    let repository = Repository::clone(
        "https://github.com/AstroTechies/UnrealModLoader.git",
        &mod_loader_dir,
    )?;

    let oid_str = "21a3723393522062ffd1af99a8ff855f20ffd53b";
    let oid = Oid::from_str(oid_str)?;
    let commit = repository.find_commit(oid)?;

    repository.branch(oid_str, &commit, false)?;

    let mut build_config = Config::new(mod_loader_dir.to_str().unwrap());

    build_config.configure_arg("-DFETCHCONTENT_QUIET=OFF");

    #[cfg(debug_assertions)]
    {
        build_config.configure_arg("-DCMAKE_BUILD_TYPE=Debug");
    }
    #[cfg(not(debug_assertions))]
    {
        build_config.configure_arg("-DCMAKE_BUILD_TYPE=Release");
    }

    let built = build_config.build();

    let build_path = built.join("build");

    #[cfg(debug_assertions)]
    let debug = true;
    #[cfg(not(debug_assertions))]
    let debug = false;

    let unreal_mod_loader_path = build_path.join("UnrealEngineModLoader");
    let unreal_mod_loader_path = match debug {
        true => unreal_mod_loader_path.join("Debug"),
        false => unreal_mod_loader_path.join("Release"),
    }
    .join("UnrealEngineModLoader.dll");

    let xinput_proxy_path = build_path.join("xinput1_3");
    let xinput_proxy_path = match debug {
        true => xinput_proxy_path.join("Debug"),
        false => xinput_proxy_path.join("Release"),
    }
    .join("xinput1_3.dll");

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
