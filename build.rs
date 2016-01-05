extern crate capnpc;

fn main() {
    ::capnpc::compile("dx16", &["src/dx16.capnp"]).unwrap();
}
