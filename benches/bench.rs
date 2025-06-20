#![feature(test)]
extern crate test;

use sirius::Sirius;
use test::Bencher;

#[derive(Sirius, PartialEq, Debug, Clone)]
struct BenchStruct {
    a: u32,
    b: String,
    c: Vec<u32>,
}

#[bench]
fn bench_serialize_u32(b: &mut Bencher) {
    let value: u32 = 123456;
    b.iter(|| {
        value.serialize_buffered();
    });
}

#[bench]
fn bench_deserialize_u32(b: &mut Bencher) {
    let value: u32 = 123456;
    let buf = value.serialize_buffered();
    b.iter(|| {
        let _ = u32::deserialize(&buf).unwrap();
    });
}

#[bench]
fn bench_serialize_string(b: &mut Bencher) {
    let value = String::from("abcdefghijklmnop");
    b.iter(|| {
        value.serialize_buffered();
    });
}

#[bench]
fn bench_deserialize_string(b: &mut Bencher) {
    let value = String::from("abcdefghijklmnop");
    let buf = value.serialize_buffered();
    b.iter(|| {
        let _ = String::deserialize(&buf).unwrap();
    });
}

#[bench]
fn bench_serialize_vec_u32(b: &mut Bencher) {
    let value: Vec<u32> = (0..100).collect();
    b.iter(|| {
        value.serialize_buffered();
    });
}

#[bench]
fn bench_deserialize_vec_u32(b: &mut Bencher) {
    let value: Vec<u32> = (0..100).collect();
    let buf = value.serialize_buffered();
    b.iter(|| {
        let _ = Vec::<u32>::deserialize(&buf).unwrap();
    });
}
