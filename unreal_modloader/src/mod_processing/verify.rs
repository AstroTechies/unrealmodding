use lazy_static::lazy_static;
use regex::Regex;

// migth need at some point
// pub fn verify_mod_id(mod_id: &str) -> bool {
//     lazy_static! {
//         static ref RE: Regex = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
//     }

//     RE.is_match(mod_id)
// }
lazy_static! {
    //                                   000 - ModName69(.author) - 1.1.1  _P.pak
    //                                   Match 1: 000
    //                                   Match 2: ModName69.author
    //                                   Match 3: 1.1.1
    pub(crate) static ref MOD_FILENAME_REGEX: Regex = Regex::new(r"(^\d{3})-([a-zA-Z0-9\.]+)-(\d+.\d+.\d+)_P.pak$").unwrap();
}

pub fn verify_mod_file_name(mod_id: &str) -> bool {
    if let Some(matches) = MOD_FILENAME_REGEX.captures(mod_id) {
        if let Some(mod_name) = matches.get(2) {
            return mod_name
                .as_str()
                .chars()
                .next()
                .map(|e| e.is_uppercase())
                .unwrap_or(false);
        }
    }

    false
}
