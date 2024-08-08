`aligned_buffer`
================

A crate for creating aligned buffers in Rust.

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
aligned_buffer = "0.1"
```

Import `aligned_buffer::prelude::*` and use the provided macro:

```rust
use aligned_buffer::prelude::*;

// This will create a buffer of 32 bytes with an alignment of 16. Additionally,
// `Buffer32` automatically implements `AlignedBuffer<32, 16>`.
#[aligned_buffer(16)]
struct Buffer32([u8; 32]);

/// This will create a buffer of 64 bytes with an alignment of 4. Additionally,
/// `Buffer64` automatically implements `AlignedBuffer<64, 4>`.
#[aligned_buffer(4)]
struct Buffer64 {
    data: [u8; 64]
}
```

The macro works on structs with either a single unnamed field or a single named
field, with any name so long as it's a valid identifier. Additionally, it's
well-behaved: any visibility modifiers or other attributes applied to the inner
field or the struct itself are preserved and will be expanded.

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](https://github.com/seancroach/aligned_buffer/blob/main/LICENSE-APACHE)
  or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](https://github.com/seancroach/aligned_buffer/blob/main/LICENSE-MIT)
  or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
