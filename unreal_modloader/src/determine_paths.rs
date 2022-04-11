use directories::BaseDirs;

use crate::AppData;

pub fn dertermine_base_path(data: &mut AppData) {
    let base_dirs = BaseDirs::new();
    if base_dirs.is_none() {
        return;
    }
    let base_dirs = base_dirs.unwrap();

    let data_dir = base_dirs.data_local_dir();
    data.base_path = Some(data_dir.join("Astro").join("Saved"));
}
