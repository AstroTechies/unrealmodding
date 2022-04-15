use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

use unreal_modintegrator::{
    metadata::{Metadata, SyncMode},
    IntegratorConfig,
};
use unreal_pak::PakFile;

mod app;
pub mod config;
mod determine_paths;
mod game_mod;
mod modconfig;
pub mod version;

use game_mod::{GameMod, GameModVersion, SelectedVersion};
use version::{GameBuild, Version};

#[derive(Debug)]
pub struct AppData {
    /// %LocalAppData%\[GameName]\Saved
    pub base_path: Option<PathBuf>,
    /// %LocalAppData%\[GameName]\Saved\Mods
    pub data_path: Option<PathBuf>,
    /// %LocalAppData%\[GameName]\Saved\Paks
    pub paks_path: Option<PathBuf>,
    /// install path
    pub install_path: Option<PathBuf>,

    pub game_build: Option<GameBuild>,

    pub game_mods: Vec<game_mod::GameMod>,
}

pub fn run<'a, C, D, T, E: std::error::Error>(config: C)
where
    D: 'static + IntegratorConfig<'a, T, E>,
    C: 'static + config::GameConfig<'a, D, T, E>,
{
    println!(
        "Got integrator config engine_version: {:?}",
        config.get_integrator_config().get_engine_version()
    );

    // TODO: remove temp test
    let mut test_versions = HashMap::new();
    test_versions.insert(
        Version::new(1, 0, 0),
        GameModVersion {
            file_name: "test_mod.pak".to_string(),
            downloaded: false,
        },
    );
    let test_mod = GameMod {
        mod_id: "TestMod".to_string(),

        versions: test_versions,
        latest_version: None,
        selected_version: SelectedVersion::Specific(Version::new(1, 0, 0)),

        active: true,

        name: "Test Mod".to_string(),
        author: "Konsti".to_string(),
        description: "test mod description".to_string(),
        game_build: GameBuild::new(1, 24, 29, 0),
        sync: SyncMode::default(),
        homepage: "https://astroneermods.space/m/TestMod".to_string(),
        download: None,
        size: 1000,
    };

    let data = Arc::new(Mutex::new(AppData {
        base_path: None,
        data_path: None,
        paks_path: None,
        install_path: None,
        game_build: None,

        game_mods: vec![test_mod],
    }));

    let should_exit = Arc::new(AtomicBool::new(false));
    let ready_exit = Arc::new(AtomicBool::new(false));
    let should_integrate = Arc::new(AtomicBool::new(false));
    let working = Arc::new(AtomicBool::new(false));

    // instantiate the GUI app
    let app = app::App {
        data: Arc::clone(&data),
        window_title: config.get_window_title(),
        dropped_files: Vec::new(),

        should_exit: Arc::clone(&should_exit),
        ready_exit: Arc::clone(&ready_exit),
        should_integrate: Arc::clone(&should_integrate),
        working: Arc::clone(&working),
    };

    // spawn a background thread to handle long running tasks
    thread::spawn(move || {
        println!("Starting background thread");

        // startup work
        working.store(true, Ordering::Relaxed);

        // get paths
        data.lock().unwrap().base_path =
            determine_paths::dertermine_base_path(config.get_game_name().as_str());
        data.lock().unwrap().install_path =
            determine_paths::dertermine_install_path(config.get_app_id());

        if data.lock().unwrap().base_path.is_some() {
            let mut data_guard = data.lock().unwrap();

            // set sub dirs
            let base_path = data_guard.base_path.as_ref().unwrap().to_owned();
            data_guard.data_path = Some(PathBuf::from(base_path.clone()).join("Mods"));
            data_guard.paks_path = Some(PathBuf::from(base_path.clone()).join("Paks"));

            // ensure the base_path/Mods directory exists
            fs::create_dir_all(data_guard.data_path.as_ref().unwrap()).unwrap();

            // TODO: better error handling for all of this
            // gather mods
            let mods_dir = fs::read_dir(data_guard.data_path.as_ref().unwrap()).unwrap();
            let mod_files: Vec<fs::DirEntry> = mods_dir
                .filter_map(|e| e.ok())
                .filter(|e| match e.file_name().into_string() {
                    Ok(s) => s.ends_with("_P.pak") && s != "999-Mods_P.pak",
                    Err(_) => false,
                })
                .collect();

            #[derive(Debug)]
            struct ReadData(String, Metadata);
            let mut mods_read: HashMap<String, Vec<ReadData>> = HashMap::new();

            // read metadata
            for file_path in mod_files.iter() {
                let file_result = (|| -> Result<(), Box<dyn Error>> {
                    let file = fs::File::open(&file_path.path())?;
                    let mut pak = PakFile::new(&file);

                    pak.load_records()?;

                    let record = &pak.read_record(&String::from("metadata.json"))?;
                    let metadata: Metadata = serde_json::from_slice(&record).unwrap();

                    let file_name = file_path.file_name().to_str().unwrap().to_owned();
                    let file_name_parts = file_name.split('_').collect::<Vec<&str>>()[0]
                        .split("-")
                        .collect::<Vec<&str>>();

                    // check that mod id in file name matches metadata
                    if file_name_parts[1] != metadata.mod_id {
                        return Err(Box::new(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "Mod id in file name does not match metadata id: {} != {}",
                                file_name_parts[1], metadata.mod_id
                            ),
                        )));
                    }

                    // check that version in file name matches metadata
                    if file_name_parts[2] != metadata.mod_version {
                        return Err(Box::new(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "Version in file name does not match metadata version: {} != {}",
                                file_name_parts[2], metadata.mod_version
                            ),
                        )));
                    }

                    let mod_id = metadata.mod_id.to_owned();

                    if !mods_read.contains_key(&mod_id) {
                        mods_read.insert(mod_id.to_owned(), Vec::new());
                    }

                    mods_read
                        .get_mut(&mod_id)
                        .unwrap()
                        .push(ReadData(file_name, metadata));

                    Ok(())
                })();
                match &file_result {
                    Ok(_) => {
                        println!(
                            "Successfully read metadata for {}",
                            file_path.file_name().to_str().unwrap()
                        );
                    }
                    Err(e) => {
                        println!(
                            "Failed to read pak file {}, error: {}",
                            file_path.file_name().to_str().unwrap(),
                            e
                        );
                    }
                }
            }

            println!("{:#?}", mods_read);

            // load config
            //modconfig::load_config(&mut *data_guard);
        }

        working.store(false, Ordering::Relaxed);

        // background loop
        loop {
            let mut data = data.lock().unwrap();
            if should_exit.load(Ordering::Relaxed) {
                println!("Background thread exiting...");
                ready_exit.store(true, Ordering::Relaxed);
                break;
            }

            if should_integrate.load(Ordering::Relaxed) && data.base_path.is_some() {
                println!("Integrating mods...");
                working.store(true, Ordering::Relaxed);
                should_integrate.store(false, Ordering::Relaxed);

                // TODO: move mods
                // TODO: run integrator

                working.store(false, Ordering::Relaxed);
            }

            drop(data);
            thread::sleep(Duration::from_millis(50));
        }
    });

    // run the GUI app
    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(Box::new(app), native_options);
}
