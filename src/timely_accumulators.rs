use std::hash::{Hash, BuildHasher};
use std::collections::HashMap;
use std::collections::hash_state::DefaultState;
use fnv::FnvHasher;

use itertools::Itertools;

pub trait SimpleAccumulator<K,V> {
    fn offer(&mut self, k: K, v: V);
    fn converge(&mut self);
    fn len(&self) -> usize;
}

pub struct HashMapAccumulator<K, V, R>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    hashmap: HashMap<K, V, DefaultState<FnvHasher>>,
    reducer: R,
}

impl<K, V, R> HashMapAccumulator<K, V, R>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    pub fn new(r: R) -> HashMapAccumulator<K, V, R> {
        HashMapAccumulator {
            hashmap: HashMap::with_hasher(DefaultState::<FnvHasher>::default()),
            reducer: r,
        }
    }
}

impl<K, V, R> SimpleAccumulator<K, V> for HashMapAccumulator<K, V, R>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    fn offer(&mut self, k: K, v: V) {
        super::crunch::aggregators::update_hashmap(&mut self.hashmap, &self.reducer, k, v);
    }
    fn converge(&mut self) {}
    fn len(&self) -> usize {
        self.hashmap.len()
    }
}

pub struct MergeSortAccumulator<K, V, R>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Ord + 'static,
          V: Send + 'static
{
    reduced: Vec<(K, V)>,
    buffer: Vec<(K, V)>,
    reducer: R,
}

impl<K, V, R> MergeSortAccumulator<K, V, R>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Ord + 'static,
          V: Send + 'static
{
    pub fn new(r: R) -> MergeSortAccumulator<K, V, R> {
        MergeSortAccumulator {
            reduced: vec![],
            buffer: Vec::with_capacity(3_000_000 + (100_000 * (super::rand::random::<usize>() % 40) ) ),
            reducer: r,
        }
    }
}

impl<K, V, R> SimpleAccumulator<K, V> for MergeSortAccumulator<K, V, R>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Ord + 'static,
          V: Send + 'static
{

    fn offer(&mut self, k: K, v: V) {
        self.buffer.push((k, v));
        if self.buffer.len() == self.buffer.capacity() {
            self.converge();
        }
    }

    fn converge(&mut self) {
        if self.buffer.len() > 0 {
            self.buffer.sort_by(|a, b| a.0.cmp(&b.0));
            let reducer = &self.reducer;
            let reduced_len = self.reduced.len();
            if reduced_len > 0 {
                let mut v = Vec::with_capacity(reduced_len + reduced_len / 5);
                {
                    let tmp = self.reduced
                                  .drain(..)
                                  .merge_by(self.buffer.drain(..), |a, b| a.0 < b.0)
                                  .coalesce(|a, b| {
                                      if a.0 == b.0 {
                                          Ok((a.0, reducer(&a.1, &b.1)))
                                      } else {
                                          Err((a, b))
                                      }
                                  });
                    v.extend(tmp);
                }
                self.reduced = v;
            } else {
                self.reduced.extend(self.buffer.drain(..)
                                  .coalesce(|a, b| {
                                      if a.0 == b.0 {
                                          Ok((a.0, reducer(&a.1, &b.1)))
                                      } else {
                                          Err((a, b))
                                      }
                                  }));
            }
        }
    }
    fn len(&self) -> usize {
        self.reduced.len()
    }
}
