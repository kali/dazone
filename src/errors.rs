extern crate glob;
extern crate csv;
extern crate capnp;
extern crate rmp;
extern crate rmp_serialize;

quick_error! {
#[derive(Debug)]
    pub enum Dx16Error {
        Io(err: ::std::io::Error) { from() }
        GlobPattern(err: glob::PatternError) { from() }
        GlobGlob(err: glob::GlobError) { from() }
        ParseInt(err: ::std::num::ParseIntError) { from() }
        ValueWrite(err: rmp::encode::ValueWriteError) { from() }
        ValueRead(err: rmp::decode::ValueReadError) { from() }
        RmpDecode(err: rmp_serialize::decode::Error) { from() }
        Capnp(err: capnp::Error) { from() }
        Csv(err: csv::Error) { from() }
        DecodeString { }
    }
}

pub type Dx16Result<T> = Result<T, Dx16Error>;
