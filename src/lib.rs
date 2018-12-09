#![feature(self_struct_ctor, trait_alias)]

// Standard
use core::{
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    ptr,
};

// Library
use fxhash::FxBuildHasher;
use allocator_api::{
    RawVec,
    alloc::CollectionAllocErr,
};
use packed_simd::u8x32;

trait RawVecGetSet<T> {
    unsafe fn get(&self, idx: usize) -> T;
    unsafe fn set(&self, idx: usize, val: T);
    unsafe fn get_ref(&self, idx: usize) -> &T;
    unsafe fn get_mut(&self, idx: usize) -> &mut T;
}

impl<T> RawVecGetSet<T> for RawVec<T> {
    unsafe fn get(&self, idx: usize) -> T {
        ptr::read((self.ptr() as *const T).add(idx))
    }

    unsafe fn set(&self, idx: usize, val: T) {
        ptr::write((self.ptr() as *mut T).add(idx), val)
    }

    unsafe fn get_ref(&self, idx: usize) -> &T {
        &*(self.ptr() as *const T).add(idx)
    }

    unsafe fn get_mut(&self, idx: usize) -> &mut T {
        &mut *(self.ptr() as *mut T).add(idx)
    }
}

pub struct HashMap<K: Hash + Eq, V, S: BuildHasher + Default = FxBuildHasher> {
    //tags: RawVec<u8x32>,
    keys: RawVec<Option<K>>,
    vals: RawVec<V>,

    len: usize, // Always <= cap
    cap: usize, // Always 2^n

    hasher: S,

    _phantom: PhantomData<K>,
}

impl<K: Hash + Eq, V, S: BuildHasher + Default> HashMap<K, V, S> {
    // Private interface

    fn tag_block_count(cap: usize) -> usize {
        (cap + 31) / 32
    }

    fn idx_for(key: &K, cap: usize, hasher: &S) -> usize {
        let mut hasher = hasher.build_hasher();
        key.hash(&mut hasher);
        hasher.finish() as usize & cap.wrapping_sub(1)
    }

    fn resize_to(&mut self, new_cap: usize) {
        assert!(new_cap.is_power_of_two());

        let mut new_keys = RawVec::with_capacity(new_cap);
        let mut new_vals = RawVec::with_capacity(new_cap);

        for new_idx in 0..new_cap {
            unsafe { new_keys.set(new_idx, None) };
        }

        // For each value in the existing map
        for idx in 0..self.cap {
            if let Some(mut key) = unsafe { self.keys.get(idx) } {
                let mut val = unsafe { self.vals.get(idx) };

                // Find free index
                let mut intended_idx = Self::idx_for(&key, new_cap, &self.hasher);
                let mut new_idx = intended_idx;
                loop {
                    match unsafe { new_keys.get_mut(idx) } {
                        None => break,
                        Some(k) => { // Robin Hood swapping
                            let other_intended_idx = Self::idx_for(k, new_cap, &self.hasher);
                            if intended_idx < other_intended_idx {
                                std::mem::swap(&mut key, k);
                                std::mem::swap(&mut val, unsafe { new_vals.get_mut(idx) });
                                intended_idx = other_intended_idx;
                            }
                        },
                    }
                    new_idx = (new_idx + 1) & new_cap.wrapping_sub(1);
                }

                // Write key and value
                unsafe {
                    new_keys.set(new_idx, Some(key));
                    new_vals.set(new_idx, val);
                }
            }
        }

        self.keys = new_keys;
        self.vals = new_vals;
        self.cap = new_cap;
    }

    fn try_grow(&mut self) {
        // Only grow if len == capacity
        if self.len < self.cap {
            return;
        }

        if self.cap == 0 {
            self.keys = RawVec::with_capacity(1);
            self.vals = RawVec::with_capacity(1);
            self.cap = 1;
            return;
        }

        self.resize_to(2 * self.cap);
    }

    fn try_shrink(&mut self) {
        // Only shrink if len <= quarter of capacity
        if self.len > self.cap / 4 {
            return;
        }

        self.resize_to(self.cap / 2);
    }

    fn get_idx(&self, key: &K) -> Option<usize> {
        let intended_idx = Self::idx_for(key, self.cap, &self.hasher);
        let mut idx = intended_idx;
        for _ in 0..self.cap {
            match unsafe { self.keys.get_ref(idx) } {
                Some(k) if k.eq(key) => return Some(idx),
                Some(k) if intended_idx < Self::idx_for(k, self.cap, &self.hasher) => return None,
                _ => {},
            }

            idx = (idx + 1) & self.cap.wrapping_sub(1);
        }
        None
    }

    // Public interface

    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, Default::default())
    }

    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        let cap = capacity.next_power_of_two();

        let keys = RawVec::with_capacity(cap);

        for idx in 0..cap {
            unsafe { keys.set(idx, None) };
        }

        Self {
            //tags: RawVec::with_capacity(Self::tag_block_count(cap)),
            keys,
            vals: RawVec::with_capacity(cap),

            len: 0,
            cap,

            hasher,

            _phantom: PhantomData,
        }
    }

    pub fn hasher(&self) -> &S {
        &self.hasher
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn reserve(&mut self, additional: usize) {
        if additional > (self.cap - self.len) {
            self.resize_to((self.len + additional).next_power_of_two());
        }
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), CollectionAllocErr> {
        self.reserve(additional);
        Ok(()) // TODO: Should this be done?
    }

    pub fn shrink_to_fit(&mut self) {
        self.resize_to(self.len.next_power_of_two());
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        if self.cap < min_capacity {
            panic!("Current capacity is smaller than supplied minimum capacity");
        }

        self.resize_to(self.len.next_power_of_two().max(min_capacity.next_power_of_two()));
    }

    pub fn keys(&self) -> Keys<K, V> {
        Keys {
            keys: &self.keys,
            vals: &self.vals,
            idx: 0,
        }
    }

    pub fn values(&self) -> Values<K, V> {
        Values {
            keys: &self.keys,
            vals: &self.vals,
            idx: 0,
        }
    }

    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut {
            keys: &self.keys,
            vals: &self.vals,
            idx: 0,
        }
    }

    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            keys: &self.keys,
            vals: &self.vals,
            idx: 0,
        }
    }

    pub fn iter_mut(&self) -> IterMut<K, V> {
        IterMut {
            keys: &self.keys,
            vals: &self.vals,
            idx: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // TODO: Drain

    pub fn clear(&mut self) {
        for idx in 0..self.cap {
            if let Some(key) = unsafe { self.keys.get_mut(idx) } {
                drop(key);
                drop(unsafe { self.vals.get_mut(idx) });
                unsafe { self.keys.set(idx, None) };
            }
        }
        self.len = 0;
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.get_idx(key).map(|idx| {
            unsafe { self.vals.get_ref(idx) }
        })
    }

    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        self.get_idx(key).map(|idx| {
            unsafe { (self.keys.get_ref(idx).as_ref().unwrap(), self.vals.get_ref(idx)) }
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get_idx(key).is_some()
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(idx) = self.get_idx(key) {
            Some(unsafe { self.vals.get_mut(idx) })
        } else {
            None
        }
    }

    pub fn insert(&mut self, mut key: K, mut val: V) -> Option<V> {
        self.try_grow();

        // Find free index
        let mut intended_idx = Self::idx_for(&key, self.cap, &self.hasher);
        let mut idx = intended_idx;
        for _ in 0..self.cap {
            match unsafe { self.keys.get_mut(idx) } {
                None => break,
                Some(k) if (*k).eq(&key) => {
                    std::mem::swap(&mut val, unsafe { self.vals.get_mut(idx) });
                    return Some(val);
                },
                Some(k) => { // Robin Hood swapping
                    let other_intended_idx = Self::idx_for(k, self.cap, &self.hasher);
                    if intended_idx < other_intended_idx {
                        std::mem::swap(&mut key, k);
                        std::mem::swap(&mut val, unsafe { self.vals.get_mut(idx) });
                        intended_idx = other_intended_idx;
                    }
                },
            }
            idx = (idx + 1) & self.cap.wrapping_sub(1);
        }

        // Write key and value
        unsafe {
            self.keys.set(idx, Some(key));
            self.vals.set(idx, val);
        }

        self.len += 1;

        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(idx) = self.get_idx(key) {
            let mut old_key = None;
            std::mem::swap(unsafe { self.keys.get_mut(idx) }, &mut old_key);
            self.try_shrink();
            Some(unsafe { self.vals.get(idx) })
        } else {
            None
        }
    }

    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        if let Some(idx) = self.get_idx(key) {
            let mut old_key = None;
            std::mem::swap(unsafe { self.keys.get_mut(idx) }, &mut old_key);
            self.try_shrink();
            Some((old_key.unwrap(), unsafe { self.vals.get(idx) }))
        } else {
            None
        }
    }

    // TODO: retain()
}

impl<K: Hash + Eq, V, S: BuildHasher + Default> Drop for HashMap<K, V, S> {
    fn drop(&mut self) {
        for idx in 0..self.cap {
            if let Some(key) = unsafe { self.keys.get_mut(idx) } {
                drop(key);
                drop(unsafe { self.vals.get_mut(idx) });
            }
        }
    }
}

impl<K: Hash + Eq + Clone, V: Clone, S: BuildHasher + Clone + Default> Clone for HashMap<K, V, S> {
    fn clone(&self) -> Self {
        //let mut tags = RawVec::with_capacity(Self::tag_block_count(self.cap));
        let mut keys = RawVec::with_capacity(self.cap);
        let mut vals = RawVec::with_capacity(self.cap);

        //for idx in 0..self.tags.cap() {
        //    unsafe { tags.set(idx, self.tags.get(idx)) };
        //}

        for idx in 0..self.cap {
            unsafe { keys.set(idx, None) };
        }

        for idx in 0..self.cap {
            if let Some(key) = unsafe { self.keys.get_ref(idx) } {
                unsafe { keys.set(idx, Some(key.clone())) };
                unsafe { vals.set(idx, self.vals.get_ref(idx).clone()) };
            }
        }

        Self {
            //tags,
            keys,
            vals,

            len: self.len,
            cap: self.cap,

            hasher: self.hasher.clone(),

            _phantom: PhantomData,
        }
    }
}

pub struct Keys<'a, K, V> {
    keys: &'a RawVec<Option<K>>,
    vals: &'a RawVec<V>,
    idx: usize,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx <= self.keys.cap() {
            self.idx += 1;
            if let Some(k) = unsafe { self.keys.get_ref(self.idx.wrapping_sub(1)) } {
                return Some(k);
            }
        }

        None
    }
}

pub struct Values<'a, K: 'a, V: 'a> {
    keys: &'a RawVec<Option<K>>,
    vals: &'a RawVec<V>,
    idx: usize,
}

impl<'a, K: 'a, V: 'a> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx <= self.keys.cap() {
            self.idx += 1;
            if unsafe { self.keys.get_ref(self.idx.wrapping_sub(1)).is_some() } {
                return Some(unsafe { self.vals.get_ref(self.idx.wrapping_sub(1)) });
            }
        }

        None
    }
}

pub struct ValuesMut<'a, K: 'a, V: 'a> {
    keys: &'a RawVec<Option<K>>,
    vals: &'a RawVec<V>,
    idx: usize,
}

impl<'a, K: 'a, V: 'a> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx <= self.keys.cap() {
            self.idx += 1;
            if unsafe { self.keys.get_ref(self.idx.wrapping_sub(1)).is_some() } {
                return Some(unsafe { self.vals.get_mut(self.idx.wrapping_sub(1)) });
            }
        }

        None
    }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    keys: &'a RawVec<Option<K>>,
    vals: &'a RawVec<V>,
    idx: usize,
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx <= self.keys.cap() {
            self.idx += 1;
            if let Some(k) = unsafe { self.keys.get_ref(self.idx.wrapping_sub(1)) } {
                return Some((
                    k,
                    unsafe { self.vals.get_ref(self.idx.wrapping_sub(1)) },
                ));
            }
        }

        None
    }
}

pub struct IterMut<'a, K: 'a, V: 'a> {
    keys: &'a RawVec<Option<K>>,
    vals: &'a RawVec<V>,
    idx: usize,
}

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx <= self.keys.cap() {
            self.idx += 1;
            if let Some(k) = unsafe { self.keys.get_ref(self.idx.wrapping_sub(1)) } {
                return Some((
                    k,
                    unsafe { self.vals.get_mut(self.idx.wrapping_sub(1)) },
                ));
            }
        }

        None
    }
}
