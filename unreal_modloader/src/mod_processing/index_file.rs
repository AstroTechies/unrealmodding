use std::collections::HashMap;

use reqwest::blocking::Client;
use unreal_modintegrator::metadata::DownloadInfo;

use crate::AppData;

pub(crate) fn gather_index_files(data: &mut AppData) -> HashMap<String, DownloadInfo> {
    let mut index_files: HashMap<String, DownloadInfo> = HashMap::new();

    for (mod_id, game_mod) in data.game_mods.iter() {
        let download_info = game_mod.download.clone();
        if let Some(download_info) = download_info {
            index_files.insert(mod_id.to_owned(), download_info);
        }
    }

    index_files
}

pub(crate) fn download_index_files(index_files: HashMap<String, DownloadInfo>) {
    let mut index_file_data: HashMap<String, String> = HashMap::new();

    let client = Client::new();

    for (mod_id, download_info) in index_files.iter() {
        println!("Downloading index file for {}", mod_id);
        let response = client.get(download_info.url.as_str()).send().unwrap();

        println!("{:?}", response);

        index_file_data.insert(mod_id.to_owned(), response.text().unwrap());
    }
}
