use crate::xoodoo::Xoodoo;

#[cfg(not(target_arch = "x86_64"))]
const N: usize = 1;
#[cfg(target_arch = "x86_64")]
const N: usize = 8;

enum Phase {
    Compress,
    Expand,
}

pub struct Xoofff {
    key: Xoodoo<1, 6>,
    state: Xoodoo<N, 6>,
    acc: Xoodoo<1, 6>,
    rc_i: usize,
    re_i: usize,
    phase: Phase,
}

impl Xoofff {
    pub fn new(key: &[u8]) -> Self {
        assert!(key.len() + 1 <= 48);
        let mut k = Xoodoo::<1, 6>::default();
        k.add_bytes(0, key, 0);
        k.add_bytes(0, &[1], key.len());
        k.permute();
        Self {
            key: k,
            state: Default::default(),
            x: Default::default(),
            phase: Phase::Compress,
        }
    }

    fn roll_compress(&mut self) -> [u8; 48] {
        // f(self.key.bytes_view(0), self.rc_i)
        self.rc_i += 1;
        panic!();
    }

    fn roll_expand(&mut self) -> [u8; 48] {
        // f(self.key.bytes_view(0), self.re_i)
        self.re_i += 1;
        panic!();
    }

    pub fn compress(&mut self, bin: &[u8], last: bool) {
        debug_assert_eq!(self.phase, Phase::Compress);
        let (chunks, rest) = bin.split_at(bin.len() % 48);
        let mut last_chunk = [0; 48];
        last_chunk[..rest.len()].copy_from_slice(rest);
        last_chunk[rest.len()] = 1;
        for chunk in chunks.chunks(48).chain(std::iter::once(&last_chunk)).enumerate() {
            let r = self.roll_compress();
            self.state.add_bytes(0, chunk ^ r, 0);
            self.state.permute();
            self.acc ^= self.state.bytes_view(0);
        }
        if last {
            let ekey = self.roll_compress();
            self.acc.permute();
            self.phase = Phase::Expand;
        }
    }

    pub fn expand(&mut self, out: &mut [u8], offset: usize, last: bool) {
        debug_assert_eq!(self.phase, Phase::Expand);
        for chunk in out.chunks(48) {
            let r = self.roll_expand();
            self.state.add_bytes(0, r, 0);
            self.state.permute();
            chunk.copy_from_slice(self.state.bytes_view() ^ ekey);
        }
        if last {
            self.acc.zeroize();
            self.phase = Phase::Compress;
        }
    }
}
