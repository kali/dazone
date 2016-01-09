use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use radix_trie::{Trie, TrieKey};
use std::sync::Mutex;

use mapred::{Aggregator, Inlet};

// ************************** HashMap Aggregator *****************************

pub struct HashMapAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    pub hashmap: Mutex<HashMap<K, V>>,
    pub reducer: &'a R,
}

impl<'a, R, K, V> HashMapAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    pub fn new(reducer: &'a R) -> HashMapAggregator<'a, R, K, V> {
        {
            HashMapAggregator {
                hashmap: Mutex::new(HashMap::new()),
                reducer: reducer,
            }
        }
    }

    pub fn as_inner(self) -> HashMap<K, V> {
        self.hashmap.into_inner().unwrap()
    }
}

impl<'a, R, K, V> Aggregator<'a, R, K, V> for HashMapAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    fn create_inlet<'b>(&'b self, i: usize) -> Box<Inlet<R, K, V> + 'b> {
        Box::new(HashMapInlet {
            parent: &self,
            partial: HashMap::new(),
            i: i,
        })
    }

    fn len(&self) -> u64 {
        self.hashmap.lock().unwrap().len() as u64
    }
}

struct HashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static,
          'a: 'b
{
    parent: &'b HashMapAggregator<'a, R, K, V>,
    partial: HashMap<K, V>,
    #[allow(dead_code)]
    i: usize,
}

impl<'a, 'b, R, K, V> Inlet<R, K, V> for HashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static,
          'a: 'b
{
    fn push_one(&mut self, k: K, v: V) {
        update_hashmap(&mut self.partial, self.parent.reducer, k, v);
    }
}

impl<'a, 'b, R, K, V> Drop for HashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    fn drop(&mut self) {
        let mut locked = self.parent.hashmap.lock().unwrap();
        for (k, v) in self.partial.drain() {
            update_hashmap(&mut locked, self.parent.reducer, k, v);
        }
    }
}

// ************************** MultiAggregator *************************

trait MultiAggregator<'a, R,K,V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static {

    fn partition(&self, k: &K) -> usize;

    fn merge<'b>(&self, inlet: &mut MultiHashMapInlet<'a, 'b, R, K, V>) {
        for (i, mut partial) in inlet.hashmaps.drain(..).enumerate() {
            self.update_partition(i, partial.drain())
        }
    }

    fn update_partition(&self, partition: usize, kvs: ::std::collections::hash_map::Drain<K, V>);
}

// ************************** MultiHashMap Aggregator *************************

pub struct MultiHashMapAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    hashmaps: Vec<Mutex<HashMap<K, V>>>,
    reducer: &'a R,
}

impl<'a, R, K, V> MultiHashMapAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    pub fn new(reducer: &'a R, partitions: usize) -> MultiHashMapAggregator<'a, R, K, V> {
        {
            MultiHashMapAggregator {
                hashmaps: (0..partitions)
                              .map(|_| Mutex::new(HashMap::new()))
                              .collect(),
                reducer: reducer,
            }
        }
    }

    pub fn as_inner(self) -> Vec<HashMap<K, V>> {
        self.hashmaps.into_iter().map(|m| m.into_inner().unwrap()).collect()
    }
}

impl<'a, R, K, V> MultiAggregator<'a, R, K, V> for MultiHashMapAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    fn update_partition(&self, partition: usize, kvs: ::std::collections::hash_map::Drain<K, V>) {
        let mut locked = self.hashmaps[partition].lock().unwrap();
        for (k, v) in kvs {
            update_hashmap(&mut locked, self.reducer, k, v);
        }
    }

    fn partition(&self, k: &K) -> usize {
        use std::hash::{Hasher, SipHasher};
        let mut s = SipHasher::new();
        k.hash(&mut s);
        s.finish() as usize % self.hashmaps.len()
    }
}

impl<'a, R, K, V> Aggregator<'a, R, K, V> for MultiHashMapAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    fn create_inlet<'b>(&'b self, i: usize) -> Box<Inlet<R, K, V> + 'b> {
        MultiHashMapInlet::new(self, self.hashmaps.len(), self.reducer, i)
    }
    fn len(&self) -> u64 {
        self.hashmaps.iter().map(|h| h.lock().unwrap().len() as u64).sum()
    }
}



// ************************** MultiTrie Aggregator *************************

pub struct MultiTrieAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static + TrieKey,
          V: Send + 'static
{
    tries: Vec<Mutex<Trie<K, V>>>,
    reducer: &'a R,
}

impl<'a, R, K, V> MultiTrieAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static + TrieKey,
          V: Send + 'static
{
    pub fn new(reducer: &'a R, partitions: usize) -> MultiTrieAggregator<'a, R, K, V> {
        {
            MultiTrieAggregator {
                tries: (0..partitions).map(|_| Mutex::new(Trie::new())).collect(),
                reducer: reducer,
            }
        }
    }

    pub fn as_inner(self) -> Vec<Trie<K, V>> {
        self.tries.into_iter().map(|m| m.into_inner().unwrap()).collect()
    }
}

impl<'a, R, K, V> MultiAggregator<'a, R, K, V> for MultiTrieAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static + TrieKey,
          V: Send + 'static
{
    fn update_partition(&self, partition: usize, kvs: ::std::collections::hash_map::Drain<K, V>) {
        let mut locked = self.tries[partition].lock().unwrap();
        for (k, v) in kvs {
            let found = locked.get_mut(&k).is_some();
            let after = if found {
                (self.reducer)(&locked.remove(&k).unwrap(), &v)
            } else {
                v
            };
            locked.insert(k, after);
        }
    }

    fn partition(&self, k: &K) -> usize {
        let coding = k.encode();
        (coding[0] as usize * 53 + coding[1] as usize) % self.tries.len()
    }
}

impl<'a, R, K, V> Aggregator<'a, R, K, V> for MultiTrieAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static + TrieKey,
          V: Send + 'static
{
    fn create_inlet<'b>(&'b self, i: usize) -> Box<Inlet<R, K, V> + 'b> {
        MultiHashMapInlet::new(self, self.tries.len(), self.reducer, i)
    }
    fn len(&self) -> u64 {
        self.tries.iter().map(|h| h.lock().unwrap().len() as u64).sum()
    }
}

// ************************** MultiHashMap Inlet *************************


struct MultiHashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static,
          'a: 'b
{
    parent: &'b MultiAggregator<'a, R, K, V>,
    hashmaps: Vec<HashMap<K, V>>,
    reducer: &'a R,
    #[allow(dead_code)]
    i: usize,
}

impl<'a, 'b, R, K, V> MultiHashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static,
          'a: 'b
{
    fn new(parent: &'b MultiAggregator<'a, R, K, V>,
           size: usize,
           reducer: &'a R,
           i: usize)
           -> Box<Inlet<R, K, V> + 'b> {
        Box::new(MultiHashMapInlet {
            parent: parent,
            hashmaps: (0..size)
                          .map(|_| HashMap::new())
                          .collect(),
            reducer: reducer,
            i: i,
        })
    }
}

impl<'a, 'b, R, K, V> Inlet<R, K, V> for MultiHashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static,
          'a: 'b
{
    fn push_one(&mut self, k: K, v: V) {
        let partial: usize = self.parent.partition(&k);
        update_hashmap(&mut self.hashmaps[partial], self.reducer, k, v)
    }
}

impl<'a, 'b, R, V, K> Drop for MultiHashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static,
          'a: 'b
{
    fn drop(&mut self) {
        self.parent.merge(self)
    }
}

fn update_hashmap<'h, 'r, R, K, V>(hash: &'h mut HashMap<K, V>, reducer: &'r R, k: K, v: V)
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    let val = hash.entry(k);
    match val {
        Entry::Occupied(prev) => {
            let next = reducer(prev.get(), &v);
            *(prev.into_mut()) = next;
        }
        Entry::Vacant(vac) => {
            vac.insert(v);
        }
    }
}

#[allow(dead_code)]
fn dump_vec_map<K, V>(i: usize, hashes: &Vec<HashMap<K, V>>)
    where K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    let lens: Vec<usize> = hashes.iter().map(|h| h.len()).collect();
    let sum: usize = lens.iter().sum();
    println!("{:4} min:{:5} max:{:5} avg:{:5}",
             i,
             lens.iter().min().unwrap(),
             lens.iter().max().unwrap(),
             sum / lens.len());
}
