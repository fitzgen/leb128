#[macro_use]
extern crate criterion;
extern crate leb128;

use criterion::Criterion;

fn write_signed(c: &mut Criterion) {
    c.bench_function("write signed", |b| {
        let mut buf = [0; 4096];

        b.iter(|| {
            let mut writable = &mut buf[..];
            for i in -1025..1025 {
                criterion::black_box(leb128::write::signed(&mut writable, i).unwrap());
            }
        })
    });
}

fn write_unsigned(c: &mut Criterion) {
    c.bench_function("write unsigned", |b| {
        let mut buf = [0; 4096];

        b.iter(|| {
            let mut writable = &mut buf[..];
            for i in 0..2050 {
                criterion::black_box(leb128::write::unsigned(&mut writable, i).unwrap());
            }
        })
    });
}

fn read_signed(c: &mut Criterion) {
    c.bench_function("read signed", |b| {
        let mut buf = [0; 4096];

        {
            let mut writable = &mut buf[..];
            for i in -1025..1025 {
                leb128::write::signed(&mut writable, i).unwrap();
            }
        }

        b.iter(|| {
            let mut readable = &buf[..];
            for _ in -1025..1025 {
                criterion::black_box(leb128::read::signed(&mut readable).unwrap());
            }
        })
    });
}

fn read_unsigned(c: &mut Criterion) {
    c.bench_function("read unsigned", |b| {
        let mut buf = [0; 4096];

        {
            let mut writable = &mut buf[..];
            for i in 0..2050 {
                leb128::write::unsigned(&mut writable, i).unwrap();
            }
        }

        b.iter(|| {
            let mut readable = &buf[..];
            for _ in 0..2050 {
                criterion::black_box(leb128::read::unsigned(&mut readable).unwrap());
            }
        })
    });
}

criterion_group!(read, read_signed, read_unsigned);
criterion_group!(write, write_signed, write_unsigned);
criterion_main!(read, write);
