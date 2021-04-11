use zeroize::Zeroize;

#[derive(Debug)]
pub struct TagMismatch;

impl std::fmt::Display for TagMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "tag verification failed.")
    }
}

impl std::error::Error for TagMismatch {}

pub const AUTH_TAG_BYTES: usize = 16;

#[derive(Clone, Debug, Default, Eq)]
pub struct Tag([u8; AUTH_TAG_BYTES]);

impl Tag {
    pub fn verify(&self, bin: [u8; AUTH_TAG_BYTES]) -> Result<(), TagMismatch> {
        if &Tag::from(bin) == self {
            Ok(())
        } else {
            Err(TagMismatch)
        }
    }
}

impl Drop for Tag {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Tag) -> bool {
        other
            .0
            .iter()
            .zip(self.0.iter())
            .fold(0, |c, (a, b)| c | (a ^ b))
            == 0
    }
}

impl AsRef<[u8]> for Tag {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for Tag {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl From<Tag> for [u8; AUTH_TAG_BYTES] {
    fn from(tag: Tag) -> Self {
        tag.0
    }
}

impl From<[u8; AUTH_TAG_BYTES]> for Tag {
    fn from(bin: [u8; AUTH_TAG_BYTES]) -> Self {
        Tag(bin)
    }
}
