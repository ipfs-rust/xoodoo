use super::{Xoodoo, ROUND_KEYS};

use core_simd::{u32x8, u8x32};
use std::mem::transmute;

const RHO_EAST_2: [u32; 32] = [
    3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10, 15, 12, 13, 14, 19, 16, 17, 18, 23, 20, 21, 22, 27, 24,
    25, 26, 31, 28, 29, 30,
];

#[inline(always)]
fn load(st: &[[u32; 12]; 8], i: usize) -> u32x8 {
    u32x8::from([
        st[0][i], st[1][i], st[2][i], st[3][i], st[4][i], st[5][i], st[6][i], st[7][i],
    ])
}

#[inline(always)]
fn store(st: &mut [[u32; 12]; 8], i: usize, v: u32x8) {
    st[0][i] = v[0];
    st[1][i] = v[1];
    st[2][i] = v[2];
    st[3][i] = v[3];
    st[4][i] = v[4];
    st[5][i] = v[5];
    st[6][i] = v[6];
    st[7][i] = v[7];
}

#[inline(always)]
fn rotate(x: u32x8, i: u32) -> u32x8 {
    x << i | x >> (32 - i)
}

#[inline(always)]
fn rotate_rho(x: u32x8) -> u32x8 {
    unsafe { transmute(transmute::<_, u8x32>(x).shuffle::<RHO_EAST_2>(Default::default())) }
}

#[inline(always)]
fn round(a: &mut [u32x8; 12], i: [usize; 12], rc: u32) {
    // Theta: Column Parity Mixer
    let mut v1 = a[3] ^ a[i[3]] ^ a[i[11]];
    let mut v2 = a[0] ^ a[i[0]] ^ a[i[8]];
    v1 = rotate(v1, 5) ^ rotate(v1, 14);
    a[0] ^= v1;
    a[i[0]] ^= v1;
    a[i[8]] ^= v1;
    v1 = a[1] ^ a[i[1]] ^ a[i[9]];
    v2 = rotate(v2, 5) ^ rotate(v2, 14);
    a[1] ^= v2;
    a[i[1]] ^= v2;
    a[i[9]] ^= v2;
    v2 = a[2] ^ a[i[2]] ^ a[i[10]];
    v1 = rotate(v1, 5) ^ rotate(v1, 14);
    a[2] ^= v1;
    a[i[2]] ^= v1;
    a[i[10]] ^= v1;
    v2 = rotate(v2, 5) ^ rotate(v2, 14);
    a[3] ^= v2;
    a[i[3]] ^= v2;
    a[i[11]] ^= v2;
    // Rho-west: Plane shift
    a[i[8]] = rotate(a[i[8]], 11);
    a[i[9]] = rotate(a[i[9]], 11);
    a[i[10]] = rotate(a[i[10]], 11);
    a[i[11]] = rotate(a[i[11]], 11);
    // Iota: round constants
    a[0] ^= u32x8::splat(rc);
    // Chi: non linear step on columns
    a[0] ^= !a[i[4]] & a[i[8]];
    a[1] ^= !a[i[5]] & a[i[9]];
    a[2] ^= !a[i[6]] & a[i[10]];
    a[3] ^= !a[i[7]] & a[i[11]];
    a[i[4]] ^= !a[i[8]] & a[0];
    a[i[5]] ^= !a[i[9]] & a[1];
    a[i[6]] ^= !a[i[10]] & a[2];
    a[i[7]] ^= !a[i[11]] & a[3];
    a[i[8]] ^= !a[0] & a[i[4]];
    a[i[9]] ^= !a[1] & a[i[5]];
    a[i[10]] ^= !a[2] & a[i[6]];
    a[i[11]] ^= !a[3] & a[i[7]];
    // Rho-east: Plane shift
    a[i[4]] = rotate(a[i[4]], 1);
    a[i[5]] = rotate(a[i[5]], 1);
    a[i[6]] = rotate(a[i[6]], 1);
    a[i[7]] = rotate(a[i[7]], 1);
    a[i[8]] = rotate_rho(a[i[8]]);
    a[i[9]] = rotate_rho(a[i[9]]);
    a[i[10]] = rotate_rho(a[i[10]]);
    a[i[11]] = rotate_rho(a[i[11]]);
}

impl Xoodoo<8, 6> {
    pub fn permute(&mut self) {
        let mut a: [u32x8; 12] = [
            load(&self.st, 0),
            load(&self.st, 1),
            load(&self.st, 2),
            load(&self.st, 3),
            load(&self.st, 4),
            load(&self.st, 5),
            load(&self.st, 6),
            load(&self.st, 7),
            load(&self.st, 8),
            load(&self.st, 9),
            load(&self.st, 10),
            load(&self.st, 11),
        ];

        round(
            &mut a,
            [6, 7, 4, 5, 5, 6, 7, 4, 8, 9, 10, 11],
            ROUND_KEYS[5],
        );
        round(
            &mut a,
            [5, 6, 7, 4, 4, 5, 6, 7, 10, 11, 8, 9],
            ROUND_KEYS[4],
        );
        round(
            &mut a,
            [4, 5, 6, 7, 7, 4, 5, 6, 8, 9, 10, 11],
            ROUND_KEYS[3],
        );
        round(
            &mut a,
            [7, 4, 5, 6, 6, 7, 4, 5, 10, 11, 8, 9],
            ROUND_KEYS[2],
        );
        round(
            &mut a,
            [6, 7, 4, 5, 5, 6, 7, 4, 8, 9, 10, 11],
            ROUND_KEYS[1],
        );
        round(
            &mut a,
            [5, 6, 7, 4, 4, 5, 6, 7, 10, 11, 8, 9],
            ROUND_KEYS[0],
        );

        store(&mut self.st, 0, a[0]);
        store(&mut self.st, 1, a[1]);
        store(&mut self.st, 2, a[2]);
        store(&mut self.st, 3, a[3]);
        store(&mut self.st, 4, a[4]);
        store(&mut self.st, 5, a[5]);
        store(&mut self.st, 6, a[6]);
        store(&mut self.st, 7, a[7]);
        store(&mut self.st, 8, a[8]);
        store(&mut self.st, 9, a[9]);
        store(&mut self.st, 10, a[10]);
        store(&mut self.st, 11, a[11]);
    }
}


impl Xoodoo<8, 12> {
    pub fn permute(&mut self) {
        let mut a: [u32x8; 12] = [
            load(&self.st, 0),
            load(&self.st, 1),
            load(&self.st, 2),
            load(&self.st, 3),
            load(&self.st, 4),
            load(&self.st, 5),
            load(&self.st, 6),
            load(&self.st, 7),
            load(&self.st, 8),
            load(&self.st, 9),
            load(&self.st, 10),
            load(&self.st, 11),
        ];

        round(
            &mut a,
            [4, 5, 6, 7, 7, 4, 5, 6, 8, 9, 10, 11],
            ROUND_KEYS[11],
        );
        round(
            &mut a,
            [7, 4, 5, 6, 6, 7, 4, 5, 10, 11, 8, 9],
            ROUND_KEYS[10],
        );
        round(
            &mut a,
            [6, 7, 4, 5, 5, 6, 7, 4, 8, 9, 10, 11],
            ROUND_KEYS[9],
        );
        round(
            &mut a,
            [5, 6, 7, 4, 4, 5, 6, 7, 10, 11, 8, 9],
            ROUND_KEYS[8],
        );
        round(
            &mut a,
            [4, 5, 6, 7, 7, 4, 5, 6, 8, 9, 10, 11],
            ROUND_KEYS[7],
        );
        round(
            &mut a,
            [7, 4, 5, 6, 6, 7, 4, 5, 10, 11, 8, 9],
            ROUND_KEYS[6],
        );
        round(
            &mut a,
            [6, 7, 4, 5, 5, 6, 7, 4, 8, 9, 10, 11],
            ROUND_KEYS[5],
        );
        round(
            &mut a,
            [5, 6, 7, 4, 4, 5, 6, 7, 10, 11, 8, 9],
            ROUND_KEYS[4],
        );
        round(
            &mut a,
            [4, 5, 6, 7, 7, 4, 5, 6, 8, 9, 10, 11],
            ROUND_KEYS[3],
        );
        round(
            &mut a,
            [7, 4, 5, 6, 6, 7, 4, 5, 10, 11, 8, 9],
            ROUND_KEYS[2],
        );
        round(
            &mut a,
            [6, 7, 4, 5, 5, 6, 7, 4, 8, 9, 10, 11],
            ROUND_KEYS[1],
        );
        round(
            &mut a,
            [5, 6, 7, 4, 4, 5, 6, 7, 10, 11, 8, 9],
            ROUND_KEYS[0],
        );

        store(&mut self.st, 0, a[0]);
        store(&mut self.st, 1, a[1]);
        store(&mut self.st, 2, a[2]);
        store(&mut self.st, 3, a[3]);
        store(&mut self.st, 4, a[4]);
        store(&mut self.st, 5, a[5]);
        store(&mut self.st, 6, a[6]);
        store(&mut self.st, 7, a[7]);
        store(&mut self.st, 8, a[8]);
        store(&mut self.st, 9, a[9]);
        store(&mut self.st, 10, a[10]);
        store(&mut self.st, 11, a[11]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r12_x8_eq_x1() {
        let mut x1_out = [0; 48];
        let mut x1 = Xoodoo::<1, 12>::default();
        x1.permute();
        x1.extract_bytes(0, &mut x1_out);

        let mut x8_out = [0; 48];
        let mut x8 = Xoodoo::<8, 12>::default();
        x8.permute();
        for i in 0..8 {
            x8.extract_bytes(i, &mut x8_out);
            println!("i {}", i);
            assert_eq!(x1_out, x8_out);
        }
    }

    #[test]
    fn test_r6_x8_eq_x1() {
        let mut x1_out = [0; 48];
        let mut x1 = Xoodoo::<1, 6>::default();
        x1.permute();
        x1.extract_bytes(0, &mut x1_out);

        let mut x8_out = [0; 48];
        let mut x8 = Xoodoo::<8, 6>::default();
        x8.permute();
        for i in 0..8 {
            x8.extract_bytes(i, &mut x8_out);
            println!("i {}", i);
            assert_eq!(x1_out, x8_out);
        }
    }
}
