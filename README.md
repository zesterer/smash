# Smash

`smash` is yet another blazingly fast hashmap written in Rust. `smash` makes use of Robin Hood hashing and the `fxhash` algorithm to achieve performance within a similar ballpark to the fastest existing hashmaps.

## Using `smash`

*`smash` is not currently hosted on [crates.io](https://crates.io) due to its early development status.*

## Performance

Here are some benchmarks demonstrating the performance of `smash` compared to existing hashmaps. Each value is in nanoseconds, although the unit isn't relevant. For each test, I've also indicated what position `smash` comes in the rankings, along with how much faster/slower it is when compared to the best existing hashmap.

```
| Test            |       std |  fxhash | hashbrown |   smash(fx) | # | speedup |
--------------------------------------------------------------------------------|
| creation        |        10 |      12 |         3 |          10 | 2 |   - 17% |
| get existing    |   297,271 |  78,504 |    64,675 |      39,295 | 1 |   + 39% |
| get nonexistent |   320,710 |  59,314 |    36,457 |      57,751 | 2 |   - 58% |
| insert          | 1,081,676 | 542,295 |   267,689 |     167,045 | 1 |   + 38% |
| remove          |   427,348 | 172,378 |   138,499 |     130,205 | 1 |   +  6% |
| iterate keys    |    38,808 |  15,360 |    31,946 |      14,752 | 1 |   +  4% |
| iterate values  |    39,318 |  15,371 |    29,442 |      20,229 | 2 |   - 32% |
```

## A Note On Hash Functions

`smash` uses the `fxhash` algorithm to compute hashes. `fxhash`, although faster than `std`'s default hashing function, is not cryptographically secure. Like `hashbrown`, and `std + fxhash`, it is possible for an attacker to design hashmap keys that produce significantly worse performance when using `smash` than the benchmarks above.

## When should I use `smash`?

**At the moment, never.**

`smash` is still under heavy development and isn't ready for production code. It's likely that it still has incorrectness bugs. That said, I'd appreciate testers to report issues very much.

However, based on current benchmarks, it's looking likely that `smash` might become a good choice in the following situations:

- When you need rapid access times for existing elements

- When you need to insert a lot of items into your hashmap

- When you need to remove a lot of items from your hashmap

`smash` is probably not such a good choice in the following situations:

- When you need to check whether a particular key exists within your hashmap a lot

## Future

Future plans for `smash` include:

- A code cleanup

- A richer API that goes beyond that which is provided by `std`'s hashmap

- More tests!

- Algorithm improvements

## License

`smash` is distributed under either of:

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at the disgression of the user.
