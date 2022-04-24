use lazy_static::lazy_static;
use regex::Regex;

// migth need at some point
// pub fn verify_mod_id(mod_id: &str) -> bool {
//     lazy_static! {
//         static ref RE: Regex = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
//     }

//     RE.is_match(mod_id)
// }

pub fn verify_mod_file_name(mod_id: &str) -> bool {
    lazy_static! {
        //                                   000  -ModName69   -1  .1  .1  _P.pak
        static ref RE: Regex = Regex::new(r"^\d{3}-[a-zA-Z0-9]+-\d+.\d+.\d+_P.pak$").unwrap();
    }

    RE.is_match(mod_id)
}
