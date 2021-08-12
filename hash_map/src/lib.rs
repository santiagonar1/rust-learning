use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem;
use std::ops::Index;
const INITIAL_BUCKETS: usize = 1;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0,
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    fn bucket<Q>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        if self.buckets.is_empty() {
            return None;
        }

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let bucket = (hasher.finish() % self.buckets.len() as u64) as usize;
        Some(bucket)
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn capacity(&self) -> usize {
        self.buckets.len()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.capacity() == 0 || self.items > (3 / 4) * self.buckets.len() {
            self.resize();
        }

        let bucket = self.bucket(&key).expect("buckets.is_empty() handled above");
        let bucket = &mut self.buckets[bucket];

        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            if ekey == &key {
                return Some(mem::replace(evalue, value));
            }
        }

        self.items += 1;
        bucket.push((key, value));
        None
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let bucket = self.bucket(key)?;
        self.buckets[bucket]
            .iter()
            .find(|&(ref ekey, _)| ekey.borrow() == key)
            .map(|&(_, ref value)| value)
    }

    fn resize(&mut self) {
        let target_size = match self.capacity() {
            0 => INITIAL_BUCKETS,
            n => 2 * n,
        };

        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));

        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[bucket].push((key, value));
        }

        self.buckets = new_buckets;
    }
}

impl<K, Q, V> Index<&Q> for HashMap<K, V>
where
    K: Borrow<Q> + Hash + Eq,
    Q: Hash + Eq,
{
    type Output = V;
    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).expect("No entry found for key")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_insert() {
        let mut map = HashMap::new();
        assert_eq!(map.insert(37, "a"), None);
        assert_eq!(map.is_empty(), false);

        map.insert(37, "b");
        assert_eq!(map.insert(37, "c"), Some("b"));
        // TODO: Indexing
        assert_eq!(map[&37], "c");
    }
}
