use fnv::FnvHasher;

#[derive(Clone)]
pub struct FnvState;

impl ::std::collections::hash_state::HashState for FnvState {
    type Hasher = FnvHasher;
    fn hasher(&self) -> FnvHasher {
        FnvHasher::default()
    }
}
