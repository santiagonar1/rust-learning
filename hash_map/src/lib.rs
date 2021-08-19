use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::iter::FromIterator;
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

pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    entry: &'a mut (K, V),
}

pub struct VacantEntry<'a, K: 'a, V: 'a> {
    key: K,
    map: &'a mut HashMap<K, V>,
    bucket: usize,
}

impl<'a, K: 'a, V: 'a> VacantEntry<'a, K, V> {
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn insert(self, value: V) -> &'a mut V {
        self.map.buckets[self.bucket].push((self.key, value));
        self.map.items += 1;
        &mut self.map.buckets[self.bucket].last_mut().unwrap().1
    }
}

impl<'a, K: 'a, V: 'a> OccupiedEntry<'a, K, V> {
    pub fn key(&self) -> &K {
        &self.entry.0
    }

    pub fn insert(self, value: V) -> &'a V {
        self.entry.1 = value;
        &self.entry.1
    }
}

pub enum Entry<'a, K: 'a, V: 'a> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V> {
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(e) => &mut e.entry.1,
            Entry::Vacant(e) => e.insert(default),
        }
    }

    pub fn or_insert_with<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Occupied(e) => &mut e.entry.1,
            Entry::Vacant(e) => e.insert(default()),
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
        Q: Hash + Eq + ?Sized,
    {
        if self.buckets.is_empty() {
            return None;
        }

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let bucket = (hasher.finish() % self.buckets.len() as u64) as usize;
        Some(bucket)
    }

    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        if self.buckets.is_empty() || self.items > (3 / 4) * self.buckets.len() {
            self.resize();
        }

        let bucket = self.bucket(&key).expect("buckets.is_empty() handled above");
        match self.buckets[bucket]
            .iter()
            .position(|&(ref ekey, _)| ekey == &key)
        {
            Some(index) => Entry::Occupied(OccupiedEntry {
                entry: &mut self.buckets[bucket][index],
            }),
            None => Entry::Vacant(VacantEntry {
                map: self,
                key,
                bucket,
            }),
        }
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
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.bucket(key)?;
        self.buckets[bucket]
            .iter()
            .find(|&(ref ekey, _)| ekey.borrow() == key)
            .map(|&(_, ref value)| value)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.bucket(key)?;
        self.buckets[bucket]
            .iter_mut()
            .find(|&&mut (ref ekey, _)| ekey.borrow() == key)
            .map(|&mut (_, ref mut value)| value)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.bucket(key)?;
        let bucket = &mut self.buckets[bucket];
        let index = bucket
            .iter()
            .position(|&(ref ekey, _)| ekey.borrow() == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(index).1)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
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
    Q: Hash + Eq + ?Sized,
{
    type Output = V;
    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).expect("No entry found for key")
    }
}

pub struct Iter<'a, K, V> {
    map: &'a HashMap<K, V>,
    bucket: usize,
    at: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => match bucket.get(self.at) {
                    Some(&(ref k, ref v)) => {
                        self.at += 1;
                        break Some((k, v));
                    }
                    None => {
                        self.bucket += 1;
                        self.at = 0;
                        continue;
                    }
                },
                None => break None,
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map: self,
            bucket: 0,
            at: 0,
        }
    }
}

pub struct IntoIter<K, V> {
    map: HashMap<K, V>,
    bucket: usize,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get_mut(self.bucket) {
                Some(bucket) => match bucket.pop() {
                    Some((k, v)) => break Some((k, v)),
                    None => {
                        self.bucket += 1;
                        continue;
                    }
                },
                None => break None,
            }
        }
    }
}

impl<K, V> IntoIterator for HashMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            map: self,
            bucket: 0,
        }
    }
}

impl<K, V> FromIterator<(K, V)> for HashMap<K, V>
where
    K: Hash + Eq,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut map = HashMap::new();
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
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

    #[test]
    fn can_remove() {
        let mut map = HashMap::new();
        assert_eq!(map.insert(37, "a"), None);
        assert_eq!(map.remove(&37), Some("a"));
        assert_eq!(map.remove(&37), None);
        assert_eq!(map.is_empty(), true);
    }

    #[test]
    fn check_contains_key() {
        let mut map = HashMap::new();
        map.insert(1, "a");
        assert_eq!(map.contains_key(&1), true);
        assert_eq!(map.contains_key(&2), false);
    }
}
