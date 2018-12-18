// Standard
use core::{
    hash::{BuildHasher, Hash, Hasher},
    ptr,
    mem,
    num::NonZeroU32,
    u64,
};

// Library
use allocator_api::{
    RawVec,
    alloc::CollectionAllocErr,
};
use fxhash::FxBuildHasher;

// Local
use crate::rewrite::raw_vec::RawVecAccess;

type HashType = NonZeroU32;

#[inline(always)]
fn idx_for(hash: HashType, cap: usize) -> usize {
    hash.get() as usize & cap.wrapping_sub(1)
}

pub struct HashMap<K: Hash + Eq, V, S: BuildHasher = FxBuildHasher> {
    hashes: RawVec<Option<HashType>>,
    keys: RawVec<K>,
    vals: RawVec<V>,

    len: usize,
    cap: usize,

    hasher_builder: S,
}

impl<K: Hash + Eq, V, S: BuildHasher> HashMap<K, V, S> {
    #[inline(always)]
    fn hash_key(&self, key: &K) -> HashType {
        let mut hasher = self.hasher_builder.build_hasher();
        key.hash(&mut hasher);
        unsafe { HashType::new_unchecked(hasher.finish() as u32 | (1 << 31)) }
    }

    #[inline(always)]
    fn raw_get<'a>(
        hashes: &'a RawVec<Option<HashType>>,
        keys: &'a RawVec<K>,
        vals: &'a RawVec<V>,
        cap: usize,
        hash: HashType,
        key: &K,
    ) -> Option<(&'a K, &'a V)> {
        let mut this_idx = idx_for(hash, cap);
        let mut idx = this_idx;
        loop {
            match unsafe { hashes.get(idx) } {
                Some(other_hash) if hash == other_hash => {
                    if unsafe { keys.entry(idx) }.eq(&key) {
                        return unsafe { Some((
                            keys.entry(idx),
                            vals.entry(idx),
                        )) };
                    }
                },
                Some(other_hash) => {
                    let other_idx = idx_for(other_hash, cap);

                    if (cap + this_idx - other_idx) & cap.wrapping_sub(1) >= cap / 2 {
                        return None;
                    }
                },
                _ => {},
            }

            idx = (idx + 1) & cap.wrapping_sub(1)
        }
    }

    #[inline(always)]
    fn raw_insert(
        hashes: &mut RawVec<Option<HashType>>,
        keys: &mut RawVec<K>,
        vals: &mut RawVec<V>,
        cap: usize,
        mut hash: HashType,
        mut key: K,
        mut val: V,
    ) -> Option<(K, V)> {
        let mut this_idx = idx_for(hash, cap);
        let mut idx = this_idx;
        loop {
            match unsafe { hashes.entry_mut(idx) } {
                None => {
                    unsafe {
                        keys.set(idx, key);
                        vals.set(idx, val);
                        hashes.set(idx, Some(hash));
                    }

                    return None;
                },
                Some(other_hash) if hash == *other_hash => {
                    if unsafe { keys.entry(idx) }.eq(&key) {
                        unsafe {
                            mem::swap(&mut key, keys.entry_mut(idx));
                            mem::swap(&mut val, vals.entry_mut(idx));
                        }

                        return Some((key, val));
                    }
                },
                Some(other_hash) => {
                    let other_idx = idx_for(*other_hash, cap);

                    if (cap + this_idx - other_idx) & cap.wrapping_sub(1) >= cap / 2 {
                        unsafe {
                            mem::swap(&mut key, keys.entry_mut(idx));
                            mem::swap(&mut val, vals.entry_mut(idx));
                            mem::swap(&mut hash, other_hash);
                        }
                        this_idx = other_idx;
                    }
                },
            }

            idx = (idx + 1) & cap.wrapping_sub(1)
        }
    }

    #[inline(always)]
    fn resize_to_cap(&mut self, new_cap: usize) {
        assert!(new_cap.is_power_of_two());
        assert!(new_cap >= self.len);

        let mut new_hashes = RawVec::with_capacity(new_cap);
        let mut new_keys = RawVec::with_capacity(new_cap);
        let mut new_vals = RawVec::with_capacity(new_cap);

        for new_idx in 0..new_cap {
            unsafe {
                new_hashes.set(new_idx, None);
            }
        }

        for idx in 0..self.cap {
            if let Some(this_hash) = unsafe { self.hashes.get(idx) } {
                let key = unsafe { self.keys.get(idx) };
                let val = unsafe { self.vals.get(idx) };

                Self::raw_insert(
                    &mut new_hashes,
                    &mut new_keys,
                    &mut new_vals,
                    new_cap,
                    this_hash,
                    key,
                    val,
                );
            }
        }

        self.hashes = new_hashes;
        self.keys = new_keys;
        self.vals = new_vals;
        self.cap = new_cap;
    }

    #[inline(always)]
    fn grow_by(&mut self, growth: usize) {
        let new_len = self.len + growth;

        if new_len * 2 > self.cap {
            self.resize_to_cap((new_len * 2).next_power_of_two());
        }
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> HashMap<K, V, S> {
    #[inline(always)]
    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        self.grow_by(1);

        let hash = self.hash_key(&key);

        self.len += 1;
        Self::raw_insert(
            &mut self.hashes,
            &mut self.keys,
            &mut self.vals,
            self.cap,
            hash,
            key,
            val,
        ).map(|(k, v)| v)
    }

    #[inline(always)]
    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash_key(&key);

        Self::raw_get(
            &self.hashes,
            &self.keys,
            &self.vals,
            self.cap,
            hash,
            key,
        ).map(|(k, v)| v)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.cap
    }
}

impl<K: Hash + Eq, V, S: BuildHasher + Default> HashMap<K, V, S> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            hashes: RawVec::new(),
            keys: RawVec::new(),
            vals: RawVec::new(),

            len: 0,
            cap: 0,

            hasher_builder: S::default(),
        }
    }
}
