use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui;
use reqwest::blocking::Client;
use unreal_modintegrator::metadata::DownloadInfo;
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

    pub game_mods: BTreeMap<String, GameMod>,
}

pub fn run<'a, C, D, T, E: std::error::Error>(config: C)
where
    D: 'static + IntegratorConfig<'a, T, E>,
    C: 'static + config::GameConfig<'a, D, T, E>,
{
    let data = Arc::new(Mutex::new(AppData {
        base_path: None,
        data_path: None,
        paks_path: None,
        install_path: None,
        game_build: None,

        game_mods: BTreeMap::new(),
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
    thread::Builder::new()
        .name("background".to_string())
        .spawn(move || {
            println!("Starting background thread");

            let start = Instant::now();

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

                let data_path = data_guard.data_path.as_ref().unwrap().to_owned();
                drop(data_guard);

                // ensure the base_path/Mods directory exists
                fs::create_dir_all(&data_path).unwrap();

                // TODO: better error handling for all of this
                // gather mods
                let mods_dir = fs::read_dir(&data_path).unwrap();

                let mod_files: Vec<fs::DirEntry> = mods_dir
                    .filter_map(|e| e.ok())
                    .filter(|e| match e.file_name().into_string() {
                        Ok(s) => s.ends_with("_P.pak") && s != "999-Mods_P.pak",
                        Err(_) => false,
                    })
                    .collect();

                // read metadata from pak files and collect for each mod_id
                let mods_read = read_pak_files(mod_files);

                let mut data_guard = data.lock().unwrap();

                // turn metadata into proper data structures
                insert_mods_from_readdata(&mods_read, &mut *data_guard);

                // set top level data
                set_mod_data_from_version(&mut *data_guard);

                // fetch index files
                let index_files = gather_index_files(&mut *data_guard);
                println!("Index files: {:#?}", index_files);
                drop(data_guard);

                download_index_files(index_files);

                let mut data_guard = data.lock().unwrap();

                // load config
                modconfig::load_config(&mut *data_guard);

                //println!("{:#?}", data_guard.game_mods);
            }

            println!(
                "Background thread startup took {} milliseconds",
                start.elapsed().as_millis()
            );

            working.store(false, Ordering::Relaxed);

            // background loop
            loop {
                if should_exit.load(Ordering::Relaxed) {
                    println!("Background thread exiting...");
                    ready_exit.store(true, Ordering::Relaxed);
                    break;
                }

                let data = data.lock().unwrap();
                if should_integrate.load(Ordering::Relaxed) && data.base_path.is_some() {
                    drop(data);
                    println!(
                        "Integrating mods with config engine_version: {:?}",
                        config.get_integrator_config().get_engine_version()
                    );

                    working.store(true, Ordering::Relaxed);
                    should_integrate.store(false, Ordering::Relaxed);

                    // TODO: move mods
                    // TODO: run integrator

                    working.store(false, Ordering::Relaxed);
                } else {
                    drop(data);
                }

                thread::sleep(Duration::from_millis(50));
            }
        })
        .expect("Failure to spawn background thread");

    // run the GUI app
    eframe::run_native(
        app.window_title.clone().as_str(),
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.iter_mut().for_each(|font| {
                font.1.tweak.scale = 1.2;
            });
            cc.egui_ctx.set_fonts(fonts);

            cc.egui_ctx.set_style(egui::Style::default());

            Box::new(app)
        }),
    );
}

#[derive(Debug)]
struct ReadData(String, Metadata);

fn read_pak_files(mod_files: Vec<fs::DirEntry>) -> HashMap<String, Vec<ReadData>> {
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

    mods_read
}

fn insert_mods_from_readdata(mods_read: &HashMap<String, Vec<ReadData>>, data: &mut AppData) {
    for (mod_id, mod_files) in mods_read.iter() {
        // check if mod is in global list, if not insert empty
        if !data.game_mods.contains_key(mod_id) {
            let game_mod = GameMod {
                versions: HashMap::new(),
                selected_version: SelectedVersion::LatestIndirect(None),

                active: false,

                name: "".to_owned(),
                author: None,
                description: None,
                game_build: None,
                sync: SyncMode::ServerAndClient,
                homepage: None,
                download: None,
                size: 0,
            };

            data.game_mods.insert(mod_id.to_owned(), game_mod);
        }

        // insert metadata
        for read_data in mod_files {
            let version = GameModVersion {
                file_name: read_data.0.clone(),
                downloaded: true,
                download_url: None,
                metadata: Some(read_data.1.clone()),
            };
            let key: Version =
                Version::try_from(&version.metadata.as_ref().unwrap().mod_version).unwrap();
            data.game_mods
                .get_mut(&version.metadata.as_ref().unwrap().mod_id)
                .unwrap()
                .versions
                .insert(key, version);
        }
    }
}

fn set_mod_data_from_version(data: &mut AppData) {
    for (_, game_mod) in data.game_mods.iter_mut() {
        // if using latest indirect, find version
        if let SelectedVersion::LatestIndirect(None) = game_mod.selected_version {
            let mut versions = game_mod.versions.keys().collect::<Vec<&Version>>();
            versions.sort();
            game_mod.selected_version =
                SelectedVersion::LatestIndirect(Some(**versions.last().unwrap()));
        }

        let use_version = match game_mod.selected_version {
            SelectedVersion::Latest(version) => version,
            SelectedVersion::Specific(version) => version,
            SelectedVersion::LatestIndirect(version) => version.unwrap(),
        };

        let version_data = game_mod.versions.get(&use_version).unwrap();
        let metadata = version_data.metadata.as_ref().unwrap();

        game_mod.name = metadata.name.to_owned();
        game_mod.author = metadata.author.to_owned();
        game_mod.description = metadata.description.to_owned();
        game_mod.game_build = match metadata.game_build {
            Some(ref game_build) => Some(GameBuild::try_from(game_build).unwrap()),
            None => None,
        };
        game_mod.sync = metadata.sync.unwrap_or(SyncMode::ServerAndClient);
        game_mod.homepage = metadata.homepage.clone();
        game_mod.download = metadata.download.clone();
        let path = data
            .data_path
            .as_ref()
            .unwrap()
            .join(version_data.file_name.clone());
        game_mod.size = fs::metadata(&path).unwrap().len();
    }
}

fn gather_index_files(data: &mut AppData) -> HashMap<String, DownloadInfo> {
    let mut index_files: HashMap<String, DownloadInfo> = HashMap::new();

    for (mod_id, game_mod) in data.game_mods.iter() {
        let download_info = game_mod.download.clone();
        if let Some(download_info) = download_info {
            index_files.insert(mod_id.to_owned(), download_info);
        }
    }

    index_files
}

fn download_index_files(index_files: HashMap<String, DownloadInfo>) {
    let mut index_file_data: HashMap<String, String> = HashMap::new();

    let client = Client::new();

    for (mod_id, download_info) in index_files.iter() {
        println!("Downloading index file for {}", mod_id);
        let response = client.get(download_info.url.as_str()).send().unwrap();

        println!("{:?}", response);

        index_file_data.insert(mod_id.to_owned(), response.text().unwrap());
    }
}
