use rawbytes::RawBytes;
use zeroize::Zeroize;

#[cfg(not(target_arch = "x86_64"))]
mod impl_portable;
#[cfg(target_arch = "x86_64")]
mod impl_x86_64;

const ROUND_KEYS: [u32; 12] = [
    0x012, 0x1a0, 0x0f0, 0x380, 0x02c, 0x060, 0x014, 0x120, 0x0d0, 0x3c0, 0x038, 0x058,
];

#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct Xoodoo {
    st: [u32; 12],
}

impl Xoodoo {
    fn bytes_view(&self) -> &[u8] {
        let view = RawBytes::bytes_view(&self.st);
        debug_assert_eq!(view.len(), 48);
        view
    }

    fn bytes_view_mut(&mut self) -> &mut [u8] {
        let view = RawBytes::bytes_view_mut(&mut self.st);
        debug_assert_eq!(view.len(), 48);
        view
    }

    fn endian_swap(&mut self) {
        for word in self.st.iter_mut() {
            *word = (*word).to_le()
        }
    }

    pub fn from_bytes(bytes: [u8; 48]) -> Self {
        let mut st = Xoodoo::default();
        let st_bytes = st.bytes_view_mut();
        st_bytes.copy_from_slice(&bytes);
        st
    }

    pub fn bytes(&self, out: &mut [u8; 48]) {
        let st_bytes = self.bytes_view();
        out.copy_from_slice(st_bytes);
    }

    pub fn add_byte(&mut self, byte: u8, offset: usize) {
        self.endian_swap();
        let st_bytes = self.bytes_view_mut();
        st_bytes[offset] ^= byte;
        self.endian_swap();
    }

    pub fn add_bytes(&mut self, bytes: &[u8]) {
        self.endian_swap();
        let st_bytes = self.bytes_view_mut();
        for (st_byte, byte) in st_bytes.iter_mut().zip(bytes) {
            *st_byte ^= byte;
        }
        self.endian_swap();
    }

    pub fn extract_bytes(&mut self, out: &mut [u8]) {
        self.endian_swap();
        let st_bytes = self.bytes_view();
        out.copy_from_slice(&st_bytes[..out.len()]);
        self.endian_swap();
    }
}

impl Drop for Xoodoo {
    fn drop(&mut self) {
        self.st.zeroize()
    }
}
