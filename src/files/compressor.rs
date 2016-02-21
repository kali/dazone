use std::io;
use snappy_framed::read::{CrcMode, SnappyFramedDecoder};
use snappy_framed::write::SnappyFramedEncoder;

#[derive(Clone,Copy)]
pub enum Compressor {
    None,
    Deflate,
    Gz,
    Snappy,
    Lz4,
}

impl Compressor {
    pub fn get(name: &str) -> Compressor {
        match name {
            "none" => Compressor::None,
            "gz" => Compressor::Gz,
            "deflate" => Compressor::Deflate,
            "snz" => Compressor::Snappy,
            "lz4" => Compressor::Lz4,
            _ => panic!("unknown compressor {}", name),
        }
    }

    pub fn for_format(name: &str) -> Compressor {
        let tokens: Vec<String> = name.split("-").map(|x| x.to_owned()).collect();
        Compressor::get(if tokens.len() == 1 {
            "none"
        } else {
            &*tokens[1]
        })
    }

    pub fn read_file<P: AsRef<::std::path::Path>>(&self, name: P) -> Box<io::Read + Send> {
        self.decompress(::std::fs::File::open(name).unwrap())
    }

    pub fn write_file<P: AsRef<::std::path::Path>>(&self, name: P) -> Box<io::Write> {
        self.compress(::std::fs::File::create(name).unwrap())
    }

    pub fn compress<W: io::Write + Send + 'static>(&self, w: W) -> Box<io::Write> {
        match *self {
            Compressor::None => Box::new(io::BufWriter::new(w)),
            Compressor::Gz => {
                Box::new(::flate2::write::GzEncoder::new(w, ::flate2::Compression::Default))
            }
            Compressor::Deflate => {
                Box::new(::flate2::write::ZlibEncoder::new(w, ::flate2::Compression::Default))
            }
            Compressor::Snappy => {
                Box::new(io::BufWriter::with_capacity(256 * 1024,
                                                      SnappyFramedEncoder::new(w).unwrap()))
            }
            Compressor::Lz4 => Box::new(::lz4::EncoderBuilder::new().build(w).unwrap()),
        }
    }

    pub fn decompress<R: io::Read + Send + 'static>(&self, r: R) -> Box<io::Read + Send> {
        match *self {
            Compressor::None => Box::new(io::BufReader::new(r)),
            Compressor::Gz => Box::new(::flate2::read::GzDecoder::new(r).unwrap()),
            Compressor::Deflate => Box::new(::flate2::read::ZlibDecoder::new(r)),
            Compressor::Snappy => Box::new(SnappyFramedDecoder::new(r, CrcMode::Ignore)),
            Compressor::Lz4 => Box::new(::lz4::Decoder::new(r).unwrap()),
        }
    }
}
