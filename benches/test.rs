#![feature(test, extern_crate_item_prelude)]

extern crate test;
extern crate smash;
extern crate fxhash;
extern crate hashbrown;

use test::{Bencher, black_box};

#[bench]
fn smash_create(b: &mut Bencher) {
    b.iter(|| black_box(smash::HashMap::<i32, i32>::new()))
}

#[bench]
fn fxhashmap_create(b: &mut Bencher) {
    b.iter(|| black_box(fxhash::FxHashMap::<i32, i32>::default()))
}

#[bench]
fn hashbrown_create(b: &mut Bencher) {
    b.iter(|| black_box(hashbrown::HashMap::<i32, i32>::default()))
}

#[bench]
fn smash_insert(b: &mut Bencher) {
    b.iter(|| {
        let mut map = smash::HashMap::<i32, i32>::new();
        for i in 0..10000 {
            map.insert(i, 10000 - i);
        }
    })
}

#[bench]
fn fxhashmap_insert(b: &mut Bencher) {
    b.iter(|| {
        let mut map = fxhash::FxHashMap::<i32, i32>::default();
        for i in 0..10000 {
            map.insert(i, 10000 - i);
        }
    })
}

#[bench]
fn hashbrown_insert(b: &mut Bencher) {
    b.iter(|| {
        let mut map = hashbrown::HashMap::<i32, i32>::new();
        for i in 0..10000 {
            map.insert(i, 10000 - i);
        }
    })
}

#[bench]
fn smash_get_in(b: &mut Bencher) {
    let mut map = smash::HashMap::<i32, i32>::new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            assert_eq!(map.get(&i), Some(10000 - i).as_ref());
        }
    })
}

#[bench]
fn fxhashmap_get_in(b: &mut Bencher) {
    let mut map = fxhash::FxHashMap::<i32, i32>::default();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            assert_eq!(map.get(&i), Some(10000 - i).as_ref());
        }
    })
}

#[bench]
fn hashbrown_get_in(b: &mut Bencher) {
    let mut map = smash::HashMap::<i32, i32>::new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            assert_eq!(map.get(&i), Some(10000 - i).as_ref());
        }
    })
}

#[bench]
fn smash_get_not_in(b: &mut Bencher) {
    let mut map = smash::HashMap::<i32, i32>::new();
    for i in 10000..20000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            assert_eq!(map.get(&i), None);
        }
    })
}

#[bench]
fn fxhashmap_get_not_in(b: &mut Bencher) {
    let mut map = fxhash::FxHashMap::<i32, i32>::default();
    for i in 10000..20000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            assert_eq!(map.get(&i), None);
        }
    })
}

#[bench]
fn hashbrown_get_not_in(b: &mut Bencher) {
    let mut map = smash::HashMap::<i32, i32>::new();
    for i in 10000..20000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            assert_eq!(map.get(&i), None);
        }
    })
}

#[bench]
fn smash_remove(b: &mut Bencher) {
    let mut map = smash::HashMap::<i32, i32>::new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        let mut map = map.clone();
        for i in 0..10000 {
            assert_eq!(map.remove(&i), Some(10000 - i));
        }
    })
}

#[bench]
fn fxhashmap_remove(b: &mut Bencher) {
    let mut map = fxhash::FxHashMap::<i32, i32>::default();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        let mut map = map.clone();
        for i in 0..10000 {
            assert_eq!(map.remove(&i), Some(10000 - i));
        }
    })
}

#[bench]
fn hashbrown_remove(b: &mut Bencher) {
    let mut map = hashbrown::HashMap::<i32, i32>::new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        let mut map = map.clone();
        for i in 0..10000 {
            assert_eq!(map.remove(&i), Some(10000 - i));
        }
    })
}
