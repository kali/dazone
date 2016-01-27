use fnv::FnvHasher;

#[derive(Clone)]
pub struct FnvState;

impl ::std::hash::BuildHasher for FnvState {
    type Hasher = FnvHasher;
    fn build_hasher(&self) -> FnvHasher {
        FnvHasher::default()
    }
}
