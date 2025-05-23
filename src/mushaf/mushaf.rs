use std::{ fmt::Display, rc::Rc };

#[cfg(feature = "colored_output")]
use colored::Colorize;

use super::Page;

/// Representation of a complete Quran Mushaf (printed copy)
///
/// This struct contains all pages and verses of the Quran with their metadata,
/// including information about line positions and page numbers.
#[derive(Clone, Debug)]
pub struct Mushaf {
    /// Number of lines per page in this Mushaf edition
    pub lines_per_page: u8,
    /// Maximum page number in this Mushaf edition
    pub max_page: u16,
    /// Collection of all pages in the Mushaf, stored as a reference-counted slice
    pub pages: Rc<[Page]>,
}

impl Mushaf {
    /// Create a new Mushaf with the specified parameters
    ///
    /// # Arguments
    /// * `lines_per_page` - Number of lines on each page in this Mushaf edition
    /// * `max_page` - Total number of pages in this Mushaf edition
    /// * `pages` - Reference-counted slice containing all pages
    pub fn new(lines_per_page: u8, max_page: u16, pages: Rc<[Page]>) -> Self {
        Self {
            lines_per_page,
            max_page,
            pages,
        }
    }

    /// Get a page by its number (1-indexed)
    ///
    /// # Arguments
    /// * `page_number` - The page number to retrieve (1-indexed)
    ///
    /// # Returns
    /// * `Some(&Page)` - Reference to the requested page if it exists
    /// * `None` - If the page number is 0 or exceeds max_page
    pub fn get_page(&self, page_number: u16) -> Option<&Page> {
        if page_number == 0 || page_number > self.max_page {
            return None;
        }

        Some(&self.pages[(page_number as usize) - 1])
    }
}

impl Display for Mushaf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Display summary info with colors if enabled
        #[cfg(feature = "colored_output")]
        {
            write!(f, "{}", "=== Mushaf Summary ===\n".bright_green().bold());
            writeln!(f, "{}: {}", "Total Pages".yellow(), self.max_page.to_string().cyan());
            writeln!(
                f,
                "{}: {}",
                "Lines per Page".yellow(),
                self.lines_per_page.to_string().cyan()
            );
            writeln!(
                f,
                "{}: {}",
                "Total Verses".yellow(),
                self.pages
                    .iter()
                    .map(|p| p.verses().len())
                    .sum::<usize>()
                    .to_string()
                    .cyan()
            )
        }

        // Plain display when colored feature is not enabled
        #[cfg(not(feature = "colored_output"))]
        {
            write!(f, "=== Mushaf Summary ===\n");
            write!(f, "Total Pages: {}\n", self.max_page);
            write!(f, "Lines per Page: {}\n", self.lines_per_page);
            write!(
                f,
                "Total Verses: {}\n",
                self.pages
                    .iter()
                    .map(|p| p.verses().len())
                    .sum::<usize>()
            )
        }
    }
}
