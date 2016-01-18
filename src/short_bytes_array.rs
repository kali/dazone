
extern crate abomonation;
use self::abomonation::Abomonation;

use std::marker::Reflect;
use std::fmt::Debug;

use std::hash::{Hash, Hasher};

pub trait ShortBytesArray : Abomonation + Hash + Send + Clone + Reflect + Eq + Debug + 'static {
    fn prefix(s: &str) -> Self;
    fn to_vec(&self) -> Vec<u8>;
}

macro_rules! short_bytes_array {
    ( $name:ident $size:expr ) => {
        #[derive(Copy,Clone,Debug,PartialEq,Eq)]
        pub struct $name([u8;$size]);

        impl ShortBytesArray for $name {
            fn prefix(s:&str) -> $name {
                let mut buf = [b' ';$size];
                {
                    use std::io::Write;
                    let mut slice:&mut [u8] = &mut buf;
                    let bytes = s.as_bytes();
                    let len = ::std::cmp::min(bytes.len(), $size);
                    slice.write_all(&bytes[0..len]).unwrap();
                }
                $name(buf)
            }

            fn to_vec(&self) -> Vec<u8> {
                self.0.to_vec()
            }
        }

        impl Abomonation for $name { }

        impl Hash for $name {
            fn hash<H>(&self, state: &mut H) where H: Hasher {
                self.0.hash(state)
            }
        }
    }
}

short_bytes_array!( K8 8 );
short_bytes_array!( K9 9 );
short_bytes_array!( K10 10 );
short_bytes_array!( K11 11 );
short_bytes_array!( K12 12 );
