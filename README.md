# sirius

A fast, zero-allocation binary serialization/deserialization library for Rust. Sirius provides a simple derive macro to automatically implement efficient binary serialization for your structs and enums, with a focus on performance and minimal overhead.

> **Note:** This crate originally started as `Schemou` in [github.com/Colabie/Colabie](https://github.com/Colabie/Colabie), but is now extracted as a standalone repository. This repo will continue to be used in `Colabie/Colabie` for serialization needs.

## Features
- **Zero-allocation**: Avoids unnecessary allocations during (de)serialization.
- **Derive macro**: Use `#[derive(Sirius)]` to auto-implement the `Sirius` trait for your types.
- **Simple API**: Serialize to any `Write`, or use `serialize_buffered()` for convenience.
- **Supports**: Structs, enums, arrays, vectors, strings, numbers, and more.

## Example
```rust
use sirius::Sirius;

#[derive(Sirius)]
struct MyStruct {
    a: u32,
    b: String,
    c: Vec<char>,
}

fn main() {
    let value = MyStruct {
        a: 42,
        b: "Hello".to_string(),
        c: vec!['H', 'i'],
    };
    let serialized = value.serialize_buffered();
    let (deserialized, bytes_read) = MyStruct::deserialize(&serialized).unwrap();
    assert_eq!(value, deserialized);
    assert_eq!(bytes_read, serialized.len());
}
```

## Benchmarks
Sirius is designed for speed. Here are real benchmark results (run on a modern x86_64 CPU, Rust nightly):

| Type                  | Sirius Serialize      | Sirius Deserialize     |
|-----------------------|----------------------|-----------------------|
| `u32`                 | ~1.14 ns/iter        | ~1.81 ns/iter         |
| `String` (16 bytes)   | ~21.17 ns/iter       | ~14.66 ns/iter        |
| `Vec<u32>` (100 items)| ~267.95 ns/iter      | ~271.41 ns/iter       |

*Note: Actual performance may vary depending on your hardware and compiler settings. For real benchmarks, run `cargo +nightly bench`.*

## License
MIT
