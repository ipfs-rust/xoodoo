use crate::xoodoo::Xoodoo;

const HASH_RATE: usize = 16;
const KEYED_ABSORB_RATE: usize = 44;
const KEYED_SQUEEZE_RATE: usize = 24;
const RATCHET_RATE: usize = 16;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Mode {
    Hash,
    Keyed,
}

impl Mode {
    #[inline(always)]
    fn absorb_rate(&self) -> usize {
        match self {
            Self::Hash => HASH_RATE,
            Self::Keyed => KEYED_ABSORB_RATE,
        }
    }

    #[inline(always)]
    fn squeeze_rate(&self) -> usize {
        match self {
            Self::Hash => HASH_RATE,
            Self::Keyed => KEYED_SQUEEZE_RATE,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Phase {
    Up,
    Down,
}

#[derive(Clone, Debug)]
pub struct Xoodyak {
    state: Xoodoo<1, 12>,
    mode: Mode,
    phase: Phase,
}

impl Xoodyak {
    pub fn hash() -> Self {
        Self {
            state: Xoodoo::default(),
            phase: Phase::Up,
            mode: Mode::Hash,
        }
    }

    pub fn keyed(
        key: &[u8],
        key_id: Option<&[u8]>,
        nonce: Option<&[u8]>,
        counter: Option<&[u8]>,
    ) -> Self {
        let mut xoodyak = Self {
            state: Xoodoo::default(),
            phase: Phase::Up,
            mode: Mode::Keyed,
        };
        xoodyak.absorb_key(key, key_id, nonce, counter);
        xoodyak
    }

    fn up(&mut self, out: Option<&mut [u8]>, cu: u8) {
        debug_assert!(
            out.as_ref().map(|b| b.len()).unwrap_or_default() <= self.mode.squeeze_rate()
        );
        self.phase = Phase::Up;
        if self.mode != Mode::Hash {
            self.state.add_bytes(0, &[cu], 47);
        }
        self.state.permute();
        if let Some(mut out) = out {
            self.state.extract_bytes(0, &mut out);
        }
    }

    fn down(&mut self, bin: Option<&[u8]>, cd: u8) {
        debug_assert!(bin.unwrap_or_default().len() <= self.mode.absorb_rate());
        self.phase = Phase::Down;
        if let Some(bin) = bin {
            self.state.add_bytes(0, &bin, 0);
            self.state.add_bytes(0, &[0x01], bin.len());
        } else {
            self.state.add_bytes(0, &[0x01], 0);
        }
        if self.mode == Mode::Hash {
            self.state.add_bytes(0, &[cd & 0x01], 47);
        } else {
            self.state.add_bytes(0, &[cd], 47);
        }
    }

    pub fn absorb(&mut self, bin: &[u8]) {
        self.absorb_any(bin, self.mode.absorb_rate(), 0x03);
    }

    pub fn absorb_more(&mut self, bin: &[u8]) {
        for chunk in bin.chunks(self.mode.absorb_rate()) {
            self.up(None, 0x00);
            self.down(Some(chunk), 0x00);
        }
    }

    fn absorb_any(&mut self, bin: &[u8], rate: usize, cd: u8) {
        let mut chunks_it = bin.chunks(rate);
        if self.phase != Phase::Up {
            self.up(None, 0x00)
        }
        self.down(chunks_it.next(), cd);
        for chunk in chunks_it {
            self.up(None, 0x00);
            self.down(Some(chunk), 0x00);
        }
    }

    fn absorb_key(
        &mut self,
        key: &[u8],
        key_id: Option<&[u8]>,
        nonce: Option<&[u8]>,
        counter: Option<&[u8]>,
    ) {
        let id_len = key_id.unwrap_or_default().len() + nonce.unwrap_or_default().len();
        let iv_len = key.len() + id_len + 1;
        assert!(iv_len <= KEYED_ABSORB_RATE);

        let mut iv = [0u8; KEYED_ABSORB_RATE];
        let (iv_key, mut rest) = iv.split_at_mut(key.len());
        iv_key.copy_from_slice(key);
        if let Some(key_id) = key_id {
            let (iv_key_id, rest2) = rest.split_at_mut(key_id.len());
            iv_key_id.copy_from_slice(key_id);
            rest = rest2;
        }
        if let Some(nonce) = nonce {
            let (iv_nonce, rest2) = rest.split_at_mut(nonce.len());
            iv_nonce.copy_from_slice(nonce);
            rest = rest2;
        }
        rest[0] = id_len as u8;
        self.absorb_any(&iv[..iv_len], KEYED_ABSORB_RATE, 0x02);
        if let Some(counter) = counter {
            self.absorb_any(counter, 1, 0x00)
        }
    }

    pub fn squeeze(&mut self, out: &mut [u8]) {
        self.squeeze_any(out, 0x40);
    }

    pub fn squeeze_key(&mut self, out: &mut [u8]) {
        debug_assert_eq!(self.mode, Mode::Keyed);
        self.squeeze_any(out, 0x20);
    }

    pub fn squeeze_more(&mut self, out: &mut [u8]) {
        for chunk in out.chunks_mut(self.mode.squeeze_rate()) {
            self.down(None, 0x00);
            self.up(Some(chunk), 0x00);
        }
    }

    pub fn squeeze_any(&mut self, out: &mut [u8], cu: u8) {
        let mut chunks_it = out.chunks_mut(self.mode.squeeze_rate());
        self.up(chunks_it.next(), cu);
        for chunk in chunks_it {
            self.down(None, 0x00);
            self.up(Some(chunk), 0x00);
        }
    }

    pub fn ratchet(&mut self) {
        debug_assert_eq!(self.mode, Mode::Keyed);
        let mut rolled_key = [0u8; RATCHET_RATE];
        self.squeeze_any(&mut rolled_key, 0x10);
        self.absorb_any(&rolled_key, self.mode.absorb_rate(), 0x00);
    }

    pub fn encrypt(&mut self, bin: &[u8], out: &mut [u8]) {
        debug_assert_eq!(self.mode, Mode::Keyed);
        let mut cu = 0x80;
        for (out_chunk, chunk) in out
            .chunks_mut(KEYED_SQUEEZE_RATE)
            .zip(bin.chunks(KEYED_SQUEEZE_RATE))
        {
            self.up(Some(out_chunk), cu);
            cu = 0x00;
            self.down(Some(chunk), 0x00);
            for (out_chunk_byte, chunk_byte) in out_chunk.iter_mut().zip(chunk) {
                *out_chunk_byte ^= *chunk_byte;
            }
        }
    }

    pub fn decrypt(&mut self, bin: &[u8], out: &mut [u8]) {
        debug_assert_eq!(self.mode, Mode::Keyed);
        let mut cu = 0x80;
        for (out_chunk, chunk) in out
            .chunks_mut(KEYED_SQUEEZE_RATE)
            .zip(bin.chunks(KEYED_SQUEEZE_RATE))
        {
            self.up(Some(out_chunk), cu);
            cu = 0x00;
            for (out_chunk_byte, chunk_byte) in out_chunk.iter_mut().zip(chunk) {
                *out_chunk_byte ^= *chunk_byte;
            }
            self.down(Some(out_chunk), 0x00);
        }
    }

    pub fn encrypt_in_place(&mut self, in_out: &mut [u8]) {
        debug_assert_eq!(self.mode, Mode::Keyed);
        let mut tmp = [0u8; KEYED_SQUEEZE_RATE];
        let mut cu = 0x80;
        for in_out_chunk in in_out.chunks_mut(KEYED_SQUEEZE_RATE) {
            self.up(Some(&mut tmp), cu);
            cu = 0x00;
            self.down(Some(in_out_chunk), 0x00);
            for (in_out_chunk_byte, tmp_byte) in in_out_chunk.iter_mut().zip(&tmp) {
                *in_out_chunk_byte ^= *tmp_byte;
            }
        }
    }

    pub fn decrypt_in_place(&mut self, in_out: &mut [u8]) {
        debug_assert_eq!(self.mode, Mode::Keyed);
        let mut tmp = [0u8; KEYED_SQUEEZE_RATE];
        let mut cu = 0x80;
        for in_out_chunk in in_out.chunks_mut(KEYED_SQUEEZE_RATE) {
            self.up(Some(&mut tmp), cu);
            cu = 0x00;
            for (in_out_chunk_byte, tmp_byte) in in_out_chunk.iter_mut().zip(&tmp) {
                *in_out_chunk_byte ^= *tmp_byte;
            }
            self.down(Some(in_out_chunk), 0x00);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyed_empty() {
        let mut st = Xoodyak::keyed(b"key", None, None, None);
        let mut out = [0u8; 32];
        st.squeeze(&mut out);
        assert_eq!(
            out,
            [
                106, 247, 180, 176, 207, 217, 130, 200, 237, 113, 163, 185, 224, 53, 120, 137, 251,
                126, 216, 3, 87, 45, 239, 214, 41, 201, 246, 56, 83, 55, 18, 108
            ]
        );
    }

    #[test]
    fn test_unkeyed_empty() {
        let mut st = Xoodyak::hash();
        let mut out = [0u8; 32];
        st.squeeze(&mut out);
        assert_eq!(
            out,
            [
                141, 216, 213, 137, 191, 252, 99, 169, 25, 45, 35, 27, 20, 160, 165, 255, 204, 246,
                41, 214, 87, 39, 76, 114, 39, 130, 131, 52, 124, 189, 128, 53
            ]
        );

        let mut st = Xoodyak::hash();
        let mut out = [0u8; 32];
        st.absorb(&[]);
        st.squeeze(&mut out);
        assert_eq!(
            out,
            [
                234, 21, 47, 43, 71, 188, 226, 78, 251, 102, 196, 121, 212, 173, 241, 123, 211, 36,
                216, 6, 232, 95, 247, 94, 227, 105, 238, 80, 220, 143, 139, 209
            ]
        );
    }

    #[test]
    fn test_encrypt() {
        let st0 = Xoodyak::keyed(b"key", None, None, None);
        let m = b"message";
        let mut c = vec![0; m.len()];
        let mut m2 = vec![0; m.len()];

        let mut st = st0.clone();
        st.encrypt(&m[..], &mut c);
        let mut st = st0.clone();
        st.decrypt(&c, &mut m2);
        assert_eq!(&m[..], m2.as_slice());

        let mut st = st0.clone();
        st.ratchet();
        st.decrypt(&c, &mut m2);
        assert_ne!(&m[..], m2.as_slice());

        let c0 = c.clone();
        let mut st = st0.clone();
        st.decrypt_in_place(&mut c);
        assert_eq!(&m[..], &c[..]);

        let mut st = st0;
        st.encrypt_in_place(&mut c);
        assert_eq!(c0, c);

        let mut tag = [0; 32];
        st.squeeze(&mut tag);
        assert_eq!(
            tag,
            [
                10, 175, 140, 82, 142, 109, 23, 111, 201, 232, 32, 52, 122, 46, 254, 206, 236, 54,
                97, 165, 40, 85, 166, 91, 124, 88, 26, 144, 100, 250, 243, 157
            ]
        );
    }

    #[test]
    fn test_unkeyed_hash() {
        let mut st = Xoodyak::hash();
        let m = b"Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.";
        st.absorb(&m[..]);
        let mut hash = [0; 32];
        st.squeeze(&mut hash);
        assert_eq!(
            hash,
            [
                144, 82, 141, 27, 59, 215, 34, 104, 197, 106, 251, 142, 112, 235, 111, 168, 19, 6,
                112, 222, 160, 168, 230, 38, 27, 229, 248, 179, 94, 227, 247, 25
            ]
        );
        st.absorb(&m[..]);
        st.squeeze(&mut hash);
        assert_eq!(
            hash,
            [
                102, 50, 250, 132, 79, 91, 248, 161, 121, 248, 225, 33, 105, 159, 111, 230, 135,
                252, 43, 228, 152, 41, 58, 242, 211, 252, 29, 234, 181, 0, 196, 220
            ]
        );
    }
}
