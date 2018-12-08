# Smash

Smash is yet another quick HashMap implementation written in Rust. Smash makes use of Robin Hood hashing and the `fxhash` hashing algorithm to achieve performance within a similar ballpark to the fastest existing hashmap implementations.

## Performance

Some benchmarks demonstrating relative between different hashmap implementations (all times are relative and are performed over a variety of configurations that are consistent between tests).

```
Test            |       std |  fxhash | hashbrown |   smash
-----------------------------------------------------------
get existing    |   299,067 |  71,630 |    39,167 |  40,741
get nonexistent |   312,143 |  55,706 |    37,318 |  81,574
insert          | 1,042,481 | 487,573 |   227,432 | 142,154
remove          |   409,926 | 144,056 |   126,923 |  79,461
iterate keys    |    37,455 |  12,902 |    29,237 |  15,263
iterate values  |    38,542 |  13,485 |    29,319 |  20,066
```

## When should I use Smash?

**At the moment, never.**

Smash is still under heavy development and isn't ready for production code. It's likely that it still has correctness bugs.

However, based on current benchmarks, it's looking likely that Smash might become a good choice in the following situations:

- When you need rapid access times for existing elements

- When you need to insert a lot of items into your hashmap

- When you need to remove a lot of items from your hashmap

Smash is probably not such a good choice in the following situations:

- When you need to check whether a particular key exists within your hashmap a lot
