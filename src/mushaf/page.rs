use std::{fmt::Debug, rc::Rc};

use super::verse::Verse;

#[cfg(feature = "colored_output")]
use colored::Colorize;

/// Representation of a single page in the Quran
///
/// Contains the page number and all verses that appear on this page,
/// with their positions and metadata.
#[derive(Default, Clone, Debug)]
pub struct Page {
    /// Page number (1-indexed)
    number: u16,
    /// Collection of verses appearing on this page, stored as a reference-counted slice
    verses: Rc<[Verse]>,
}

impl Page {
    /// Create a new Page with the specified parameters
    ///
    /// # Arguments
    /// * `number` - Page number (1-indexed)
    /// * `verses` - Reference-counted slice containing all verses on this page
    ///
    /// # Returns
    /// A new Page instance
    #[must_use]
    pub const fn new(number: u16, verses: Rc<[Verse]>) -> Self {
        Self { number, verses }
    }

    /// Get the page number
    ///
    /// # Returns
    /// The page number (1-indexed)
    #[must_use]
    pub const fn number(&self) -> u16 {
        self.number
    }

    /// Get all verses on this page
    ///
    /// # Returns
    /// A slice containing all verses on this page
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn verses(&self) -> &[Verse] {
        &self.verses
    }

    pub fn total_verses(&self) -> usize {
        self.verses.len()
    }
}

// Add Display implementation for Page
impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "colored_output")]
        {
            writeln!(f, "Page {}", self.number.to_string().yellow().bold())?;

            for (i, verse) in self.verses.iter().enumerate() {
                writeln!(f, "  {:2}- {}", i + 1, verse)?;
            }
        }

        #[cfg(not(feature = "colored_output"))]
        {
            write!(f, "Page {}\n", self.number)?;

            for (i, verse) in self.verses.iter().enumerate() {
                write!(f, "  {}. {}\n", i + 1, verse)?;
            }
        }

        Ok(())
    }
}
