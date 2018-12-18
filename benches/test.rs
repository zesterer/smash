#![feature(test, extern_crate_item_prelude)]

extern crate test;
extern crate smash;
extern crate fxhash;
extern crate hashbrown;

use test::{Bencher, black_box};

use std::collections::hash_map::RandomState;

fn smashmap_new() -> smash::HashMap<i32, i32> {
    //smash::HashMap::<i32, i32, RandomState>::with_capacity_and_hasher(0, RandomState::new())
    smash::HashMap::<i32, i32>::new()
}

#[bench]
fn smash_create(b: &mut Bencher) {
    b.iter(|| black_box(smashmap_new()))
}

#[bench]
fn std_create(b: &mut Bencher) {
    b.iter(|| black_box(std::collections::HashMap::<i32, i32>::new()))
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
        let mut map = smashmap_new();
        for i in 0..10000 {
            map.insert(i, 10000 - i);
        }
        black_box(map);
    })
}

#[bench]
fn std_insert(b: &mut Bencher) {
    b.iter(|| {
        let mut map = std::collections::HashMap::<i32, i32>::new();
        for i in 0..10000 {
            map.insert(i, 10000 - i);
        }
        black_box(map);
    })
}

#[bench]
fn fxhashmap_insert(b: &mut Bencher) {
    b.iter(|| {
        let mut map = fxhash::FxHashMap::<i32, i32>::default();
        for i in 0..10000 {
            map.insert(i, 10000 - i);
        }
        black_box(map);
    })
}

#[bench]
fn hashbrown_insert(b: &mut Bencher) {
    b.iter(|| {
        let mut map = hashbrown::HashMap::<i32, i32>::new();
        for i in 0..10000 {
            map.insert(i, 10000 - i);
        }
        black_box(map);
    })
}

#[bench]
fn smash_get_in(b: &mut Bencher) {
    let mut map = smashmap_new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            let val = map.get(&i);
            assert_eq!(val, Some(10000 - i).as_ref());
            black_box(val);
        }
    })
}

#[bench]
fn smash_rewrite_get_in(b: &mut Bencher) {
    let mut map = smash::rewrite::HashMap::<i32, i32>::new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            let val = map.get(&i);
            assert_eq!(val, Some(10000 - i).as_ref());
            black_box(val);
        }
    })
}

#[bench]
fn std_get_in(b: &mut Bencher) {
    let mut map = std::collections::HashMap::<i32, i32>::new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            let val = map.get(&i);
            assert_eq!(val, Some(10000 - i).as_ref());
            black_box(val);
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
            let val = map.get(&i);
            assert_eq!(val, Some(10000 - i).as_ref());
            black_box(val);
        }
    })
}

#[bench]
fn hashbrown_get_in(b: &mut Bencher) {
    let mut map = hashbrown::HashMap::<i32, i32>::new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            let val = map.get(&i);
            assert_eq!(val, Some(10000 - i).as_ref());
            black_box(val);
        }
    })
}

#[bench]
fn smash_get_not_in(b: &mut Bencher) {
    let mut map = smashmap_new();
    for i in 10000..20000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            let val = map.get(&i);
            assert_eq!(val, None);
            black_box(val);
        }
    })
}

#[bench]
fn std_get_not_in(b: &mut Bencher) {
    let mut map = std::collections::HashMap::<i32, i32>::new();
    for i in 10000..20000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            let val = map.get(&i);
            assert_eq!(val, None);
            black_box(val);
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
            let val = map.get(&i);
            assert_eq!(val, None);
            black_box(val);
        }
    })
}

#[bench]
fn hashbrown_get_not_in(b: &mut Bencher) {
    let mut map = hashbrown::HashMap::<i32, i32>::new();
    for i in 10000..20000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        for i in 0..10000 {
            let val = map.get(&i);
            assert_eq!(val, None);
            black_box(val);
        }
    })
}

#[bench]
fn smash_remove(b: &mut Bencher) {
    let mut map = smashmap_new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        let mut map = map.clone();
        for i in 0..10000 {
            let val = map.remove(&i);
            assert_eq!(val, Some(10000 - i));
            black_box(val);
        }
    })
}

#[bench]
fn std_remove(b: &mut Bencher) {
    let mut map = std::collections::HashMap::<i32, i32>::new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        let mut map = map.clone();
        for i in 0..10000 {
            let val = map.remove(&i);
            assert_eq!(val, Some(10000 - i));
            black_box(val);
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
            let val = map.remove(&i);
            assert_eq!(val, Some(10000 - i));
            black_box(val);
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
            let val = map.remove(&i);
            assert_eq!(val, Some(10000 - i));
            black_box(val);
        }
    })
}

#[bench]
fn smash_iter_keys(b: &mut Bencher) {
    let mut map = smashmap_new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        assert_eq!(map.keys().map(|i| *i as u64).sum::<u64>(), (0..10000).sum::<u64>());
    });
    black_box(map);
}

#[bench]
fn std_iter_keys(b: &mut Bencher) {
    let mut map = std::collections::HashMap::<i32, i32>::new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        assert_eq!(map.keys().map(|i| *i as u64).sum::<u64>(), (0..10000).sum::<u64>());
    });
    black_box(map);
}

#[bench]
fn fxhashmap_iter_keys(b: &mut Bencher) {
    let mut map = fxhash::FxHashMap::<i32, i32>::default();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        assert_eq!(map.keys().map(|i| *i as u64).sum::<u64>(), (0..10000).sum::<u64>());
    });
    black_box(map);
}

#[bench]
fn hashbrown_iter_keys(b: &mut Bencher) {
    let mut map = hashbrown::HashMap::<i32, i32>::new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        assert_eq!(map.keys().map(|i| *i as u64).sum::<u64>(), (0..10000).sum::<u64>());
    });
    black_box(map);
}

#[bench]
fn smash_iter_values(b: &mut Bencher) {
    let mut map = smashmap_new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        assert_eq!(map.values().map(|i| *i as u64).sum::<u64>(), (0..10000).map(|i| 10000 - i).sum::<u64>());
    });
    black_box(map);
}

#[bench]
fn std_iter_values(b: &mut Bencher) {
    let mut map = std::collections::HashMap::<i32, i32>::new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        assert_eq!(map.values().map(|i| *i as u64).sum::<u64>(), (0..10000).map(|i| 10000 - i).sum::<u64>());
    });
    black_box(map);
}

#[bench]
fn fxhashmap_iter_values(b: &mut Bencher) {
    let mut map = fxhash::FxHashMap::<i32, i32>::default();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        assert_eq!(map.values().map(|i| *i as u64).sum::<u64>(), (0..10000).map(|i| 10000 - i).sum::<u64>());
    });
    black_box(map);
}

#[bench]
fn hashbrown_iter_values(b: &mut Bencher) {
    let mut map = hashbrown::HashMap::<i32, i32>::new();
    for i in 0..10000 {
        map.insert(i, 10000 - i);
    }
    b.iter(|| {
        assert_eq!(map.values().map(|i| *i as u64).sum::<u64>(), (0..10000).map(|i| 10000 - i).sum::<u64>());
    });
    black_box(map);
}
