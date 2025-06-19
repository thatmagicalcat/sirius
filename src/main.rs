#![feature(test)]

extern crate test;

use sirius::Sirius;
use test::Bencher;
use bitcode::{Encode, Decode};

#[derive(Sirius, Encode, Decode, Debug, PartialEq, Eq)]
struct Person {
    name: String,
    age: u8,
    known_languages: Vec<String>,
}

fn make_big_dataset(n: usize) -> Person {
    let mut v = vec![];
    for _ in 0..n {
        v.extend_from_slice(&[
            "Rust".repeat(n),
            "C".repeat(n),
            "Assembly".repeat(n),
            "Python".repeat(n),
        ]);
    }

    Person {
        name: "person".repeat(n),
        age: 69,
        known_languages: v,
    }
}

#[bench]
fn bench_sirius_serialize(b: &mut Bencher) {
    let data = make_big_dataset(10);
    let mut output = Vec::with_capacity(1024 * 1024); // pre-allocate 1MB buffer

    b.iter(|| {
        let _ = data.serialize(&mut output);
        output.clear();
    });
}

#[bench]
fn bench_sirius_deserialize(b: &mut Bencher) {
    let data = make_big_dataset(10);
    let serialized = data.serialize_buffered();

    b.iter(|| {
        let _ = Person::deserialize(&serialized).unwrap();
    });
}

#[bench]
fn bench_bitcode_serialize(b: &mut Bencher) {
    let data = make_big_dataset(10);

    b.iter(|| {
        let _ = bitcode::encode(&data);
    });
}

#[bench]
fn bench_bitcode_deserialize(b: &mut Bencher) {
    let data = make_big_dataset(10);
    let serialized = bitcode::encode(&data);

    b.iter(|| {
        let _: Person = bitcode::decode(&serialized).unwrap();
    });
}

fn main() {}
