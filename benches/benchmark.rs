use criterion::{criterion_group, criterion_main, Criterion};
use xoodyak::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Xoodoo permutation", |b| {
        let mut out = [0u8; 48];
        let mut st = Xoodoo::<1, 12>::default();
        b.iter(|| {
            st.permute();
            st.extract_bytes(0, &mut out);
            out
        })
    });

    c.bench_function("Xoodyak hash", |b| {
        let mut out = [0u8; 64];
        let mut st = Xoodyak::hash();
        b.iter(|| {
            st.absorb(b"Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. ");
            st.squeeze(&mut out);
            out
        })
    });

    c.bench_function("Xoodyak keyed", |b| {
        let mut out = [0u8; 64];
        let mut st = Xoodyak::keyed(b"key", None, None, None);
        b.iter(|| {
            st.absorb(b"Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. ");
            st.squeeze(&mut out);
            out
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
