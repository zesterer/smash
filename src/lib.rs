#![feature(self_struct_ctor, trait_alias)]

// Standard
use core::{
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    ptr,
};

// Library
use fxhash::FxHasher;
use allocator_api::RawVec;

unsafe fn rawvec_add<T>(vec: &RawVec<T>, idx: usize) -> &mut T {
    &mut *(vec.ptr() as *mut T).add(idx)
}

trait RawVecGetSet<T> {
    unsafe fn get(&self, idx: usize) -> T;
    unsafe fn set(&mut self, idx: usize, val: T);
    unsafe fn get_ref(&self, idx: usize) -> &T;
    unsafe fn get_mut(&mut self, idx: usize) -> &mut T;
}

impl<T> RawVecGetSet<T> for RawVec<T> {
    unsafe fn get(&self, idx: usize) -> T {
        ptr::read((self.ptr() as *const T).add(idx))
    }

    unsafe fn set(&mut self, idx: usize, val: T) {
        ptr::write((self.ptr() as *mut T).add(idx), val)
    }

    unsafe fn get_ref(&self, idx: usize) -> &T {
        &*(self.ptr() as *const T).add(idx)
    }

    unsafe fn get_mut(&mut self, idx: usize) -> &mut T {
        &mut *(self.ptr() as *mut T).add(idx)
    }
}

pub struct HashMap<K: Hash + PartialEq + Clone, V: Clone> {
    keys: RawVec<Option<K>>,
    vals: RawVec<V>,

    len: usize, // Always <= cap
    cap: usize, // Always 2^n

    _phantom: PhantomData<K>,
}

impl<K: Hash + PartialEq + Clone, V: Clone> HashMap<K, V> {
    pub fn new() -> Self {
        Self {
            keys: RawVec::new(),
            vals: RawVec::new(),

            len: 0,
            cap: 0,

            _phantom: PhantomData,
        }
    }

    fn idx_for(key: &K, cap: usize) -> usize {
        let mut hasher = FxHasher::default();
        key.hash(&mut hasher);
        hasher.finish() as usize & cap.wrapping_sub(1)
    }

    fn resize_to(&mut self, new_cap: usize) {
        let mut new_keys = RawVec::with_capacity(new_cap);
        let mut new_vals = RawVec::with_capacity(new_cap);

        for new_idx in 0..new_cap {
            unsafe { new_keys.set(new_idx, None) };
        }

        // For each value in the existing map
        for idx in 0..self.cap {
            unsafe {
                if let Some(key) = self.keys.get(idx) {
                    // Find free index
                    let mut new_idx = Self::idx_for(&key, new_cap);
                    while new_keys.get_ref(new_idx).is_some() {
                        new_idx = (new_idx + 1) & new_cap.wrapping_sub(1);
                    }

                    // Write key and value
                    new_keys.set(new_idx, Some(key));
                    new_vals.set(new_idx, self.vals.get(idx));
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

    pub fn insert(&mut self, key: K, mut val: V) -> Option<V> {
        self.try_grow();

        // Find free index
        let mut idx = Self::idx_for(&key, self.cap);
        for _ in 0..self.cap {
            match unsafe { self.keys.get_ref(idx) } {
                None => break,
                Some(k) if k.eq(&key) => {
                    std::mem::swap(&mut val, unsafe { self.vals.get_mut(idx) });
                    return Some(val);
                },
                Some(k) => {},
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
        // Find free index
        let mut idx = Self::idx_for(&key, self.cap);
        for _ in 0..self.cap {
            match unsafe { self.keys.get_ref(idx) } {
                None => break,
                Some(k) if k.eq(&key) => {
                    let val = unsafe {
                        self.keys.set(idx, None);
                        self.vals.get(idx)
                    };
                    self.try_shrink();
                    return Some(val);
                },
                Some(k) => {},
            }
            idx = (idx + 1) & self.cap.wrapping_sub(1);
        }

        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut idx = Self::idx_for(key, self.cap);
        for _ in 0..self.cap {
            if unsafe { self.keys.get(idx).map(|k| k.eq(key)).unwrap_or(false) } {
                return Some(unsafe { self.vals.get_ref(idx) });
            }
            idx = (idx + 1) & self.cap.wrapping_sub(1);
        }
        None
    }
}

impl<K: Hash + PartialEq + Clone, V: Clone> Drop for HashMap<K, V> {
    fn drop(&mut self) {
        for idx in 0..self.cap {
            if let Some(key) = unsafe { self.keys.get_mut(idx) } {
                drop(key);
                drop(unsafe { self.vals.get_mut(idx) });
            }
        }
    }
}

impl<K: Hash + PartialEq + Clone, V: Clone> Clone for HashMap<K, V> {
    fn clone(&self) -> Self {
        let mut keys = RawVec::with_capacity(self.cap);
        let mut vals = RawVec::with_capacity(self.cap);

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
            keys,
            vals,

            len: self.len,
            cap: self.cap,

            _phantom: PhantomData,
        }
    }
}
