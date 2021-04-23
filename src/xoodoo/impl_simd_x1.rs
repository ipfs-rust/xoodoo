use super::{Xoodoo, ROUND_KEYS};

use core_simd::{u32x4, u8x16};
use std::mem::transmute;

const RHO_EAST_2: [u32; 16] = [11, 8, 9, 10, 15, 12, 13, 14, 3, 0, 1, 2, 7, 4, 5, 6];

impl<const R: usize> Xoodoo<1, R> {
    #[allow(clippy::many_single_char_names)]
    pub fn permute(&mut self) {
        let st = &mut self.st[0];
        let mut a: u32x4 = u32x4::from([st[0], st[1], st[2], st[3]]);
        let mut b: u32x4 = u32x4::from([st[4], st[5], st[6], st[7]]);
        let mut c: u32x4 = u32x4::from([st[8], st[9], st[10], st[11]]);
        for &round_key in ROUND_KEYS[..R].iter().rev() {
            let mut p: u32x4 = (a ^ b ^ c).shuffle::<{ [3, 0, 1, 2] }>(Default::default());
            let mut e: u32x4 = (p << 5) | (p >> (32 - 5));
            p = (p << 14) | (p >> (32 - 14));
            e ^= p;
            a ^= e;
            b ^= e;
            c ^= e;
            b = b.shuffle::<{ [3, 0, 1, 2] }>(Default::default());
            c = (c << 11) | (c >> (32 - 11));
            a ^= u32x4::from([round_key, 0, 0, 0]);
            a ^= !b & c;
            b ^= !c & a;
            c ^= !a & b;
            b = (b << 1) | (b >> (32 - 1));
            c = unsafe {
                transmute(transmute::<_, u8x16>(c).shuffle::<{ RHO_EAST_2 }>(Default::default()))
            };
        }
        st[0..4].copy_from_slice(&a[..]);
        st[4..8].copy_from_slice(&b[..]);
        st[8..12].copy_from_slice(&c[..]);
    }
}
