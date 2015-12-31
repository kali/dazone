extern crate glob;
extern crate rmp;
#[macro_use] extern crate quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum Dx16Error {
        Io(err: std::io::Error) { from() }
        GlobPattern(err: glob::PatternError) { from() }
        GlobGlob(err: glob::GlobError) { from() }
        ParseInt(err: std::num::ParseIntError) { from() }
        ValueWrite(err: rmp::encode::ValueWriteError) { from() }
    }
}

pub type Dx16Result<T> = Result<T, Dx16Error>;

pub fn data_dir_for(state:&str, set:&str, table:&str) -> String {
    format!("data/{}/{}/{}", state, set, table)
}

