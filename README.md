# Smash

`smash` is yet another blazingly fast hashmap written in Rust. `smash` makes use of Robin Hood hashing and the `fxhash` algorithm to achieve performance within a similar ballpark to the fastest existing hashmaps.

## Using `smash`

*`smash` is not currently hosted on [crates.io](https://crates.io) due to its early development status.*

## Performance

Here are some benchmarks demonstrating the performance of `smash` compared to existing hashmaps. Each value is in nanoseconds, although the unit isn't relevant. For each test, I've also indicated what position `smash` comes in the rankings, along with how much faster/slower it is when compared to the best existing hashmap.

```
| Test            |       std |  fxhash | hashbrown |   smash | # | diff |
--------------------------------------------------------------------------
| get existing    |   281,400 |  71,551 |    37,835 |  34,525 | 1 | +10% |
| get nonexistent |   312,143 |  55,706 |    37,318 |  81,574 | 3 | -54% |
| insert          | 1,042,481 | 487,573 |   227,432 | 142,154 | 1 | +60% |
| remove          |   409,926 | 144,056 |   126,923 |  79,461 | 1 | +60% |
| iterate keys    |    37,455 |  12,902 |    29,237 |  15,263 | 2 | -15% |
| iterate values  |    38,542 |  13,485 |    29,319 |  20,066 | 2 | -33% |
```

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
