use std::cmp::Ordering;

#[cfg(feature = "colored_output")]
use colored::Colorize;

/// Information about a single verse (ayah) in the Quran
///
/// This struct contains metadata about a verse, including its sura and number,
/// its position on the page, and how many lines it spans.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Verse {
    /// Number of the Sura (1-114) containing this verse
    pub sura: u8,
    /// Number of the verse within its sura
    pub number: u16,
    /// Position on the page as (x position 0.0-1.0, line number 1-15)
    pub position: (f32, u8),
    /// Number of lines this verse spans (can be fractional)
    pub lines: f32,
}

// Implement Eq - we can do this because we've already implemented PartialEq
// This means that if two verses have the same sura, number, position, and lines,
// they are considered equal
impl Eq for Verse {}

// Implement Hash for using Verse in HashMaps or HashSets
impl std::hash::Hash for Verse {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // We only hash based on sura and number since position and lines
        // are f32 which doesn't implement Hash
        self.sura.hash(state);
        self.number.hash(state);
    }
}

// Add Display implementation with conditional colored output
impl std::fmt::Display for Verse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "colored_output")]
        {
            write!(
                f,
                "{} {:6} {} {:4} {} {:12} {} {}",
                "Sura:".black(),
                self.sura.to_string().yellow().bold(),
                "Ayah:".black(),
                self.number.to_string().yellow().bold(),
                "Position:".black(),
                format!("({:.1}, {:.1})", self.position.0, self.position.1).magenta(),
                "Lines:".black(),
                self.lines.to_string().magenta()
            )
        }

        #[cfg(not(feature = "colored_output"))]
        {
            write!(f, "Sura {}:{}", self.sura, self.number)
        }
    }
}

// Implement some useful methods for the Verse struct
impl Verse {
    /// Create a new Verse with the specified parameters
    ///
    /// # Arguments
    /// * `sura` - Sura number (1-114)
    /// * `number` - Verse number within the sura
    /// * `position` - Position on the page as (relative position 0.0-1.0, line number)
    /// * `lines` - Number of lines this verse spans (can be fractional)
    ///
    /// # Returns
    /// A new Verse instance
    pub fn new(sura: u8, number: u16, position: (f32, u8), lines: f32) -> Self {
        Self {
            sura,
            number,
            position,
            lines,
        }
    }

    /// Check if the verse is the last of the page
    ///
    /// # Returns
    /// `true` if the verse is the last of the page, `false` otherwise
    pub fn is_last_of_page(&self) -> bool {
        self.position.1 == 15
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_last_of_page() {
        let verse = Verse::new(1, 1, (0.0, 15), 1.0);
        assert!(verse.is_last_of_page());
    }

    #[test]
    fn test_is_not_last_of_page() {
        for i in 1..15 {
            let verse = Verse::new(1, 1, (0.0, i), 1.0);
            assert!(!verse.is_last_of_page());
        }
    }
}
