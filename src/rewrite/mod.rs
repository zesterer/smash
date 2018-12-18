mod raw_vec;
pub mod hashmap;

// Standard
use core::{
    hash::{BuildHasher, Hash, Hasher},
    ptr,
    mem,
    ops::{Sub, BitXor},
};

// Library
use allocator_api::{
    RawVec,
    alloc::CollectionAllocErr,
};
use fxhash::FxBuildHasher;
use packed_simd::{u32x16, m32x16};

// Local
use self::raw_vec::RawVecAccess;

// In each lane, the first bit indicates presence.
// The remaining 31 bits are a subset of the hash and are used for indexing
// PXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
type SimdHashes = u32x16;
type MSimdHashes = m32x16;
const HASH_LEN: usize = 32;
const HASH_LANES: usize = SimdHashes::lanes();
const MAX_CAP: usize = 1 << (HASH_LEN - 1);
const PRESENT_MASK: u32 = 1 << (HASH_LEN as u32 - 1);
const HASH_MASK: u32 = !PRESENT_MASK;

fn simds_for_cap(cap: usize) -> usize {
    // TODO: Should this be enabled in optimized builds?
    // TODO: This needs pre-pooping
    debug_assert!(cap <= MAX_CAP);

    (cap + HASH_LANES - 1) / HASH_LANES
}

pub struct HashMap<K, V, S = FxBuildHasher> {
    hash_vec: RawVec<SimdHashes>,
    key_vec: RawVec<K>,
    val_vec: RawVec<V>,

    len: usize,
    cap: usize,

    hasher_builder: S,
}

// Internal methods
impl<K: Hash + Eq, V, S: BuildHasher> HashMap<K, V, S> {
    #[inline(always)]
    fn hash_for(&self, key: &K) -> u32 {
        let mut hasher = self.hasher_builder.build_hasher();
        key.hash(&mut hasher);
        hasher.finish() as u32 | PRESENT_MASK
    }

    /*
    // TODO: Implement this perhaps? More complex than non-SIMD
    fn simd_contains_swap_for(simd: &SimdHashes, cap: usize, hash: u32) -> bool {
        let cap_mask = HASH_MASK & cap.wrapping_sub(1);

        let results = SimdHashes::splat(cap as u32 + hash & cap_mask)
            .sub(simd
                .and(SimdHashes::splat(cap_mask))
            );

        let present = simd.and(SimdHashes::splat(PRESENT_MASK));

        SimdHashes::from_cast(
            MSimdHashes::from_cast(present)
                .select(results, MSimdHashes::splat(0))
        ).max_element() > cap / 2
    }
    */

    #[inline(always)]
    fn raw_get<'a>(
        hash_vec: &'a RawVec<SimdHashes>,
        key_vec: &'a RawVec<K>,
        val_vec: &'a RawVec<V>,
        hash: u32,
        key: &K,
    ) -> Option<(&'a K, &'a V)> {
        let cap = key_vec.cap();
        let desired_idx = ((hash & HASH_MASK) as usize) & cap.wrapping_sub(1);

        // Shortcut for 0 collisions
        if unsafe { hash_vec.entry(desired_idx / HASH_LANES).extract_unchecked(desired_idx % HASH_LANES) } == hash {
            if unsafe { key_vec.entry(desired_idx) }.eq(key) {
                return unsafe { Some((
                    key_vec.entry(desired_idx),
                    val_vec.entry(desired_idx),
                )) };
            }
        }

        let mut simd_idx = desired_idx / HASH_LANES;
        let simd_cap = simds_for_cap(cap);
        for _ in 0..simd_cap {
            let simd = unsafe { hash_vec.entry(simd_idx) };

            if simd.bitxor(SimdHashes::splat(hash)).min_element() == 0 {
                for lane in 0..HASH_LANES {
                    if unsafe { simd.extract_unchecked(lane) } == hash {
                        let idx = simd_idx * HASH_LANES + lane;

                        if unsafe { key_vec.entry(idx) }.eq(key) {
                            return unsafe { Some((
                                key_vec.entry(idx),
                                val_vec.entry(idx),
                            )) };
                        }
                    }
                };
            }

            simd_idx = (simd_idx + 1) & simd_cap.wrapping_sub(1);
        }

        None
    }

    #[inline(always)]
    fn raw_insert(
        hash_vec: &mut RawVec<SimdHashes>,
        key_vec: &mut RawVec<K>,
        val_vec: &mut RawVec<V>,
        hash: u32,
        mut key: K,
        mut val: V,
    ) -> Option<(K, V)> {
        let cap = key_vec.cap();
        let desired_idx = ((hash & HASH_MASK) as usize) & cap.wrapping_sub(1);

        let mut idx = desired_idx;
        for _ in 0..cap {
            let (simd_idx, lane) = (idx / HASH_LANES, idx % HASH_LANES);

            let simd = unsafe { hash_vec.entry_mut(simd_idx) };

            let other_hash = simd.extract(lane);

            // TODO: Robin Hood hashing

            if other_hash & PRESENT_MASK == 0 {
                *simd = simd.replace(lane, hash);
                unsafe {
                    key_vec.set(idx, key);
                    val_vec.set(idx, val);
                }
                return None;
            } else if other_hash == hash {
                if unsafe { key_vec.entry(idx).eq(&key) } {
                    unsafe {
                        mem::swap(&mut key, key_vec.entry_mut(idx));
                        mem::swap(&mut val, val_vec.entry_mut(idx));
                    }
                    return Some((key, val));
                }
            }

            idx = (idx + 1) & cap.wrapping_sub(1);
        }

        // TODO: Handle this a little better
        panic!("RawVec is full!");

        /*
        let mut simd_idx = desired_idx / HASH_LANES;
        for _ in 0..hash_vec.cap() {
            if Self::simd_contains_swap_for(hash_vec.entry(simd_idx), cap, hash) {
                // Perform swap
            } else {

            }

            simd_idx = (simd_idx + 1) & hash_vec.cap().wrapping_sub(1);
        }
        */
    }

    #[inline(always)]
    fn resize_to_capacity(&mut self, new_cap: usize) {
        // TODO: This needs pre-pooping
        assert!(new_cap.is_power_of_two());
        assert!(new_cap >= self.len);

        // Make sure we actually need to do something
        if new_cap == self.capacity() {
            return;
        }

        // Create new RawVecs to move everything in to
        let mut new_hash_vec = RawVec::with_capacity(simds_for_cap(new_cap));
        let mut new_key_vec = RawVec::with_capacity(new_cap);
        let mut new_val_vec = RawVec::with_capacity(new_cap);

        // Clear out the hash_vec to indicate a lack of entries
        for nidx in 0..new_hash_vec.cap() {
            unsafe { new_hash_vec.set(nidx, SimdHashes::splat(0)); }
        }

        // Go through the current entries, inserting them into the new RawVecs
        for osimd_idx in 0..self.hash_vec.cap() {
            let simd = unsafe { self.hash_vec.entry(osimd_idx) };

            // If no blocks have present entries, skip to the next one
            // TODO: Benchmark whether removing this is performant
            if simd.max_element() & PRESENT_MASK == 0 {
                continue;
            }

            // For each hash in the SIMD block, insert it into the new RawVecs
            for lane in 0..HASH_LANES {
                let hash = unsafe { simd.extract(lane) };

                // If the entry is actually present
                if hash & PRESENT_MASK != 0 {
                    let oidx = osimd_idx * HASH_LANES + lane;
                    Self::raw_insert(
                        &mut new_hash_vec,
                        &mut new_key_vec,
                        &mut new_val_vec,
                        hash,
                        unsafe { self.key_vec.get(oidx) },
                        unsafe { self.val_vec.get(oidx) },
                    );
                }
            }
        }

        self.hash_vec = new_hash_vec;
        self.key_vec = new_key_vec;
        self.val_vec = new_val_vec;

        self.cap = new_cap;
    }

    #[inline(always)]
    fn grow_by(&mut self, growth: usize) {
        let new_len = self.len + growth;

        // Only grow if the new length exceeds the current capacity
        if new_len > self.capacity() {
            self.resize_to_capacity(new_len.next_power_of_two());
        }
    }
}

// Public interface
impl<K: Hash + Eq, V, S: BuildHasher> HashMap<K, V, S> {
    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        self.grow_by(1);

        self.len += 1;

        let hash = self.hash_for(&key);

        Self::raw_insert(
            &mut self.hash_vec,
            &mut self.key_vec,
            &mut self.val_vec,
            hash,
            key,
            val,
        ).map(|(k, v)| v)
    }

    #[inline(always)]
    pub fn get(&self, key: &K) -> Option<&V> {
        Self::raw_get(
            &self.hash_vec,
            &self.key_vec,
            &self.val_vec,
            self.hash_for(&key),
            key,
        ).map(|(k, v)| v)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        return self.len
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        // Capacity is determined by the capacity of key_vec (although val_vec should be the same)
        return self.cap
    }

    pub fn dbg(&self) {
        println!("-----");
        for i in 0..self.capacity() {
            println!("{} = {:08X}", i, unsafe { self.hash_vec.entry(i / HASH_LANES).extract(i % HASH_LANES) });
        }
    }
}

impl<K: Hash + Eq, V, S: BuildHasher + Default> HashMap<K, V, S> {
    pub fn new() -> Self {
        Self {
            hash_vec: RawVec::new(),
            key_vec: RawVec::new(),
            val_vec: RawVec::new(),

            len: 0,
            cap: 0,

            hasher_builder: S::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    fn construct() {
        let hm: HashMap<i32, i32> = HashMap::new();
    }

    #[test]
    fn test_simds_for_cap() {
        // Ensure that we have the right number of SimdHashes for a given capacity
        assert_eq!(simds_for_cap(0), 0);
        assert_eq!(simds_for_cap(1), 1);
        assert_eq!(simds_for_cap(SimdHashes::lanes()), 1);
        assert_eq!(simds_for_cap(SimdHashes::lanes() - 1), 1);
        assert_eq!(simds_for_cap(SimdHashes::lanes() * 13), 13);
        assert_eq!(simds_for_cap(SimdHashes::lanes() * 13 + 1), 14);
    }

    #[test]
    fn test_simds_for_cap_max() {
        simds_for_cap(MAX_CAP);
    }

    #[test]
    #[should_panic]
    fn test_simds_for_cap_exceed_max() {
        simds_for_cap(MAX_CAP + 1);
    }

    #[test]
    fn test_insert_get_sum() {
        let mut hm = HashMap::<u32, u32>::new();
        let nums = (0..10000).map(|i| i * 100 + rand::random::<u32>() % 100).collect::<Vec<_>>();

        let mut sum = 0;
        for (i, &num) in nums.iter().enumerate() {
            assert_eq!(hm.len(), i);
            assert_eq!(hm.capacity().max(1), i.next_power_of_two());

            let val = rand::random::<u32>() % 10000;
            hm.insert(num, val);
            sum += val;
        }

        let mut new_sum = 0;
        for num in nums {
            new_sum += hm.get(&num).unwrap();
        }

        assert_eq!(sum, new_sum);
    }
}
