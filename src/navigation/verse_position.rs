use crate::{ mushaf::Verse, navigation::LookupError };
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct VersePosition(u8, u16);

impl VersePosition {
    /// Create a new `VersePosition`
    ///
    /// # Arguments
    /// * `sura` - Sura number (1-114)
    /// * `verse` - Verse number within the sura (1-286)
    ///
    /// # Errors
    /// * `LookupError::InvalidSura` if the sura number is invalid
    /// * `LookupError::InvalidVerse` if the verse number is invalid
    pub fn new(sura: u8, verse: u16) -> Self {
        Self(sura, verse)
    }

    /// Create a new `VersePosition` for the start of the Mushaf
    pub const fn start() -> Self {
        Self(1, 1)
    }
    /// Create a new `VersePosition` for the end of the Mushaf
    pub const fn end() -> Self {
        Self(114, 6)
    }

    #[must_use]
    pub const fn sura(&self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn verse(&self) -> u16 {
        self.1
    }

    #[must_use]
    pub const fn tuple(&self) -> (u8, u16) {
        (self.0, self.1)
    }
}

impl Default for VersePosition {
    #[must_use]
    fn default() -> Self {
        Self(1, 1)
    }
}

// Implement equality with `Verse`
impl PartialEq<Verse> for VersePosition {
    fn eq(&self, other: &Verse) -> bool {
        self.0 == other.sura && self.1 == other.number
    }
}

// From Verse
impl From<Verse> for VersePosition {
    fn from(verse: Verse) -> Self {
        Self(verse.sura, verse.number)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_eq() {
        assert_eq!(VersePosition::start(), VersePosition::start());
    }

    #[test]
    fn test_ne() {
        assert_ne!(VersePosition::start(), VersePosition::new(1, 2));
    }

    #[test]
    fn invalid_sura() {
        assert!(VersePosition::new(0, 1) == VersePosition::new(0, 1));
    }
}
