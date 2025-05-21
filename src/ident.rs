/// The length of a MavLink parameters identifier/path
const MAX_NAMED_LEN: usize = 16;

/// Describes the identifier/path of a parameter
///
/// This is designed to be fully compatible with the MavLink
/// parameter protocol, by being a 16-byte null-terminated String
/// To get a utf8 string slice (`&str`), use [`Ident::as_str`]
/// and for the null-terminated 16-byte buffer, use [`Ident::as_raw`].
#[derive(Clone, PartialEq, Debug)]
pub struct Ident {
    buf: [u8; MAX_NAMED_LEN],
    len: usize,
}

impl Ident {
    pub fn new() -> Self {
        Ident {
            buf: [b'\0'; MAX_NAMED_LEN],
            len: 0,
        }
    }

    /// Expose the inner string slice
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).expect("Invalid utf8")
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
        } else if self.len + entry.len() + 1 <= MAX_NAMED_LEN {
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
            *self = Ident::new()
        }
    }
}

#[cfg(test)]
mod tests {
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
