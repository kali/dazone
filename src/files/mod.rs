extern crate flate2;
extern crate glob;

use std::path;

pub mod cap;
pub mod rmp;
pub mod csv;

pub fn data_dir_for(state: &str, set: &str, table: &str) -> String {
    format!("data/{}/{}/{}", state, set, table)
}

pub fn files_for_format(set: &str, table: &str, format: &str) -> Vec<path::PathBuf> {
    let source_root = data_dir_for(format, set, table);
    let glob = source_root.clone() + "/*." + format;
    let mut vec: Vec<path::PathBuf> = glob::glob(&glob)
                                          .unwrap()
                                          .map(|p| p.unwrap().to_owned())
                                          .collect();
    vec.sort();
    vec
}
