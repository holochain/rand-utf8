# rand-utf8

Random utf8 utility. This crate is `#![no_std]` but requires `alloc`.

#### Example

```rust
let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
let my_str = rand_utf8::rand_utf8(&mut rng, 32);
assert_eq!(32, my_str.as_bytes().len());
```

License: MIT/Apache-2.0
