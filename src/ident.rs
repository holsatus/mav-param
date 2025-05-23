/// The length of a MavLink parameters identifier/path
const MAX_NAMED_LEN: usize = 16;

/// Describes the identifier/path of a parameter
///
/// This is designed to be fully compatible with the MavLink
/// parameter protocol, by being a 16-byte null-terminated String
///
/// To get a utf8 string slice (`&str`), use [`Ident::as_str`]
/// and for the null-terminated 16-byte buffer, use [`Ident::as_raw`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    buf: [u8; MAX_NAMED_LEN],
    len: usize,
}

impl core::fmt::Debug for Ident {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Ident")
            .field("buf.as_str()", &self.as_str())
            .field("buf", &self.buf)
            .field("len", &self.len)
            .finish()
    }
}

impl TryFrom<&[u8]> for Ident {
    type Error = crate::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // Find the null-byte
        let bytes = match value.iter().position(|b| *b == b'\0') {
            Some(pos) if pos <= MAX_NAMED_LEN => pos,
            _ if value.len() <= MAX_NAMED_LEN => value.len(),
            _ => return Err(crate::Error::SequenceTooLong),
        };

        // This is safety-critical as it ensures the buffer contains valid utf8.
        let Ok(string) = core::str::from_utf8(&value[..bytes]) else {
            return Err(crate::Error::SequenceNotUtf8);
        };

        let mut ident = Ident::new();
        ident.buf[..bytes].copy_from_slice(string.as_bytes());
        ident.len = bytes;
        Ok(ident)
    }
}

impl<const N: usize> TryFrom<&[u8; N]> for Ident {
    type Error = crate::Error;

    fn try_from(value: &[u8; N]) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl Default for Ident {
    fn default() -> Self {
        Ident::new()
    }
}

impl Ident {
    /// Creates a new empty identifier.
    ///
    /// The identifier is initialized with null bytes and zero length.
    #[must_use]
    pub fn new() -> Self {
        Ident {
            buf: [b'\0'; MAX_NAMED_LEN],
            len: 0,
        }
    }

    /// Creates a new identifier using up to 16 characters oif the provided `&str`.
    #[must_use]
    pub fn from_str_truncated(string: &str) -> Self {
        let mut ident = Self::new();
        let amount = string.len().min(MAX_NAMED_LEN);
        ident.buf[..amount].copy_from_slice(&string.as_bytes()[..amount]);
        ident.len = amount;
        ident
    }

    /// Expose the inner string slice
    pub fn as_str(&self) -> &str {
        // It is fine to unwrap since we always
        // only push valid utf8 to the buffer
        let result = core::str::from_utf8(&self.buf[..self.len]);
        debug_assert!(result.is_ok());

        // SAFETY: In all places we add data to the buffer,
        // it is either &str or validated utf8.
        unsafe { result.unwrap_unchecked() }
    }

    /// Expose the inner null-terminated string. Compatible with MavLink parameter names
    pub fn as_raw(&self) -> &[u8; MAX_NAMED_LEN] {
        &self.buf
    }

    /// Add an entry to the buffer
    pub(crate) fn push_entry(&mut self, entry: &str) -> bool {
        // Check if we have space for the segment + separator
        if self.len == 0 && entry.len() <= MAX_NAMED_LEN {
            self.buf[..entry.len()].copy_from_slice(entry.as_bytes());
            self.len = entry.len();
            true
        } else if self.len + entry.len() < MAX_NAMED_LEN {
            self.buf[self.len] = b'.';
            self.len += 1;
            self.buf[self.len..(self.len + entry.len())].copy_from_slice(entry.as_bytes());
            self.len += entry.len();
            true
        } else {
            false
        }
    }

    /// Remove the last entry from the buffer
    pub(crate) fn pop_entry(&mut self) {
        // Find the last period and truncate there
        if let Some(pos) = self.buf[..self.len].iter().rev().position(|b| *b == b'.') {
            (self.len - pos - 1..self.len).for_each(|idx| self.buf[idx] = b'\0');
            self.len -= pos + 1;
        } else {
            *self = Ident::new();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn from_str_truncated() {
        // Basic back and forth conversion
        let ident = super::Ident::from_str_truncated("hello.world");
        assert_eq!("hello.world", ident.as_str());

        // Basic back and forth conversion
        let ident = super::Ident::from_str_truncated("hello.world.foo.bar");
        assert_eq!("hello.world.foo.", ident.as_str());
    }

    #[test]
    fn try_from() {
        // Basic back and forth conversion
        let ident = super::Ident::try_from(b"hello.world").unwrap();
        assert_eq!("hello.world", ident.as_str());

        // find null-byte
        let ident = super::Ident::try_from(b"hello.world\0....").unwrap();
        assert_eq!("hello.world", ident.as_str());

        // find null-byte (with trailing invalid utf8)
        let ident = super::Ident::try_from(b"hello.world\0\xE0\xA0").unwrap();
        assert_eq!("hello.world", ident.as_str());

        // Reject invalid utf8
        assert_eq!(
            super::Ident::try_from(b"hello.world\xE0\xA0"),
            Err(crate::Error::SequenceNotUtf8)
        );

        // Reject too long strings
        assert_eq!(
            super::Ident::try_from(b"hello.world.foo.bar"),
            Err(crate::Error::SequenceTooLong)
        );
    }

    #[test]
    fn push_pop_single() {
        let mut ident = super::Ident::new();

        ident.push_entry("root");
        assert_eq!(ident.as_str(), "root");

        ident.pop_entry();
        assert_eq!(ident.as_str(), "");
    }

    #[test]
    fn push_with_dot() {
        let mut ident = super::Ident::new();

        ident.push_entry("root.");
        assert_eq!(ident.as_str(), "root.");

        ident.pop_entry();
        assert_eq!(ident.as_str(), "root");
    }

    #[test]
    fn push_pop_multi() {
        let mut ident = super::Ident::new();

        ident.push_entry("root");
        assert_eq!(ident.as_str(), "root");

        ident.push_entry("mod");
        assert_eq!(ident.as_str(), "root.mod");

        ident.push_entry("leaf");
        assert_eq!(ident.as_str(), "root.mod.leaf");

        ident.pop_entry();
        assert_eq!(ident.as_str(), "root.mod");

        ident.pop_entry();
        assert_eq!(ident.as_str(), "root");

        ident.pop_entry();
        assert_eq!(ident.as_str(), "");
    }

    #[test]
    fn push_limits() {
        let mut ident = super::Ident::new();

        // 15 characters
        assert_eq!(ident.push_entry("xxxxxxxxxxxxxxx"), true);
        assert_eq!(ident.as_str().len(), 15);
        ident.pop_entry();

        // 16 characters
        assert_eq!(ident.push_entry("xxxxxxxxxxxxxxxx"), true);
        assert_eq!(ident.as_str().len(), 16);
        ident.pop_entry();

        // 17 characters (fails)
        assert_eq!(ident.push_entry("xxxxxxxxxxxxxxxxx"), false);
        assert_eq!(ident.as_str().len(), 0);
    }
}
