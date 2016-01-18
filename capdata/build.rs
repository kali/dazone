extern crate capnpc;

fn main() {
    ::capnpc::compile("dazone", &["src/dazone.capnp"]).unwrap();
}
