#![feature(test, extern_crate_item_prelude)]

extern crate test;
extern crate smash as _smash;

use test::{Bencher, black_box};
use rand::prelude::*;

use _smash::rewrite::hashmap as smash;

#[inline(always)]
fn new_hashmap() -> smash::HashMap<u32, u32> {
    smash::HashMap::<u32, u32>::new()
}

fn new_random_set(n: usize) -> Vec<(u32, u32)> {
    let mut nums = std::collections::HashMap::new();
    while nums.len() < n {
        nums.insert(
            rand::random::<u32>(),
            rand::random::<u32>(),
        );
    }
    nums.into_iter().collect()
}

#[bench]
fn perf_smash_construct(b: &mut Bencher) {
    b.iter(|| black_box(new_hashmap()));
}

#[bench]
fn perf_smash_insert(b: &mut Bencher) {
    let mut hm = new_hashmap();
    let set = new_random_set(10000);

    b.iter(|| {
        for (a, b) in set.iter() {
            black_box(hm.insert(*a, *b));
        }
    });
}

#[bench]
fn perf_smash_get(b: &mut Bencher) {
    let mut hm = new_hashmap();
    let set = new_random_set(10000);

    for (a, b) in set.iter() {
        hm.insert(*a, *b);
    }

    b.iter(|| {
        for (a, b) in set.iter() {
            assert_eq!(hm.get(&a), Some(b));
        }
    });
}