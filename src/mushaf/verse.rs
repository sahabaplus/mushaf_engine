use std::cmp::Ordering;

#[cfg(feature = "colored_output")]
use colored::Colorize;

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Verse {
    pub sura: u8,
    pub number: u16,
    pub position: (f32, u8), // (0.0-1.0, 1-15)
    pub lines: f32,
}

// Implement Eq - we can do this because we've already implemented PartialEq
// This means that if two verses have the same sura, number, position, and lines,
// they are considered equal
impl Eq for Verse {}

// Implement Ord for full comparison capability
impl Ord for Verse {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare sura
        match self.sura.cmp(&other.sura) {
            Ordering::Equal => {
                // If sura is the same, compare ayah number
                self.number.cmp(&other.number)
            }
            ordering => ordering,
        }
    }
}

// Implement Hash for using Verse in HashMaps or HashSets
impl std::hash::Hash for Verse {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // We only hash based on sura and number since position and lines
        // are f32 which doesn't implement Hash
        self.sura.hash(state);
        self.number.hash(state);

        // If you want to hash position and lines, you'd need to convert them
        // For example:
        self.position.0.to_bits().hash(state);
        self.position.1.hash(state);
        self.lines.to_bits().hash(state);
    }
}

// Add Display implementation with conditional colored output
impl std::fmt::Display for Verse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "colored_output")]
        {
            write!(
                f,
                "{} {:3} {} {:4} {} {:12} {} {}",
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
    pub fn new(sura: u8, number: u16, position: (f32, u8), lines: f32) -> Self {
        Self {
            sura,
            number,
            position,
            lines,
        }
    }
}
