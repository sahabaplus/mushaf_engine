use std::{ fmt::Debug, rc::Rc };

use super::verse::Verse;

#[cfg(feature = "colored_output")]
use colored::Colorize;

/// Representation of a single page in the Quran
///
/// Contains the page number and all verses that appear on this page,
/// with their positions and metadata.
#[derive(Default, Clone)]
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
    pub fn new(number: u16, verses: Rc<[Verse]>) -> Page {
        Page {
            number,
            verses,
        }
    }

    /// Get the page number
    ///
    /// # Returns
    /// The page number (1-indexed)
    pub fn number(&self) -> u16 {
        self.number
    }

    /// Get all verses on this page
    ///
    /// # Returns
    /// A slice containing all verses on this page
    pub fn verses(&self) -> &[Verse] {
        &self.verses
    }
}

#[cfg(not(feature = "colored_output"))]
impl std::fmt::Debug for Verse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Verse")
            .field("sura", &self.sura)
            .field("number", &self.number)
            .field("position", &self.position)
            .field("lines", &self.lines)
            .finish()
    }
}

#[cfg(feature = "colored_output")]
impl Debug for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}\n", "Page:".bright_green().bold(), self.number.to_string().yellow());

        write!(f, "\t{}\n", "Verses:".bright_green().bold());
        for (i, verse) in self.verses.iter().enumerate() {
            write!(f, "\t\t{:2}- {}\n", (i + 1).to_string().blue(), verse);
        }

        Ok(())
    }
}
