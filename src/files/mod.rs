extern crate flate2;
extern crate glob;

use std::{path, process};
use std::io;

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

#[cfg(target_os="macos")]
fn gzcat() -> &'static str {
    "gzcat"
}

#[cfg(not(target_os="macos"))]
fn gzcat() -> &'static str {
    "zcat"
}

pub struct PipeReader {
    child: process::Child,
}

impl io::Read for PipeReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.child.stdout.as_mut().unwrap().read(buf);
        if let Ok(0) = res {
            try!(self.child.wait());
        }
        res
    }
}

pub fn gz_read<P: AsRef<path::Path>>(file:P) -> PipeReader {
    let child = process::Command::new(gzcat())
        .arg("-d")
        .arg(file.as_ref().as_os_str())
        .stdout(process::Stdio::piped())
        .spawn()
        .unwrap();
    PipeReader { child: child }
}

pub fn zpipe_read<P: AsRef<path::Path>>(file:P) -> PipeReader {
    let child = process::Command::new("./zpipe.sh")
        .arg(file.as_ref().as_os_str())
        .stdout(process::Stdio::piped())
        .spawn()
        .unwrap();
    PipeReader { child: child }
}
