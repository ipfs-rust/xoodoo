use rawbytes::RawBytes;
use zeroize::Zeroize;

mod impl_simd_x1;
mod impl_simd_x8;

const ROUND_KEYS: [u32; 12] = [
    0x012, 0x1a0, 0x0f0, 0x380, 0x02c, 0x060, 0x014, 0x120, 0x0d0, 0x3c0, 0x038, 0x058,
];

/// Xoodoo permutation parameterized over the number of states and the number of rounds. Typical
/// R values are 6 for xoofff and 12 for xoodyak, and typical values for N would be 1 for xoodyak
/// and depending on if the cpu supports 128/256/512 vector instructions 4/8/16.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Xoodoo<const N: usize, const R: usize> {
    st: [[u32; 12]; N],
}

impl<const N: usize, const R: usize> Default for Xoodoo<N, R> {
    fn default() -> Self {
        Self { st: [[0; 12]; N] }
    }
}

impl<const N: usize, const R: usize> Xoodoo<N, R> {
    fn bytes_view(&self, i: usize) -> &[u8] {
        let view = RawBytes::bytes_view(&self.st[i]);
        debug_assert_eq!(view.len(), 48);
        view
    }

    fn bytes_view_mut(&mut self, i: usize) -> &mut [u8] {
        let view = RawBytes::bytes_view_mut(&mut self.st[i]);
        debug_assert_eq!(view.len(), 48);
        view
    }

    fn endian_swap(&mut self) {
        for state in self.st.iter_mut() {
            for word in state.iter_mut() {
                *word = (*word).to_le();
            }
        }
    }

    pub fn zeroize(&mut self) {
        for i in 0..N {
            self.st[i].zeroize()
        }
    }

    pub fn add_bytes(&mut self, i: usize, bytes: &[u8], offset: usize) {
        self.endian_swap();
        let st_bytes = self.bytes_view_mut(i);
        for (st_byte, byte) in st_bytes.iter_mut().skip(offset).zip(bytes) {
            *st_byte ^= byte;
        }
        self.endian_swap();
    }

    pub fn extract_bytes(&mut self, i: usize, out: &mut [u8]) {
        self.endian_swap();
        let st_bytes = self.bytes_view(i);
        out.copy_from_slice(&st_bytes[..out.len()]);
        self.endian_swap();
    }
}

impl<const N: usize, const R: usize> Drop for Xoodoo<N, R> {
    fn drop(&mut self) {
        self.zeroize()
    }
}
