use crate::mushaf::Verse;

#[cfg(feature = "colored_output")]
use colored::Colorize;

/// Result of a navigation operation that exceeds the boundaries of a page or sura
///
/// When navigation goes beyond available lines, this structure captures information
/// about the overflow including how many lines exceeded the boundary and which verse
/// was reached at the boundary.
#[derive(Debug, Clone)]
pub struct OverflowResult {
    /// Number of lines that overflowed beyond the boundary
    pub overflow_lines: f32,
    /// Reference to the verse at the boundary where overflow occurred
    pub overflowed_verse: Verse,
}

impl OverflowResult {
    /// Create a new `OverflowResult` with specified parameters
    ///
    /// # Arguments
    /// * `overflow_lines` - Number of lines that overflowed beyond the boundary
    /// * `overflowed_verse` - Reference to the verse at the boundary where overflow occurred
    ///
    /// # Returns
    /// A new `OverflowResult` instance
    #[must_use]
    pub const fn new(overflow_lines: f32, overflowed_verse: Verse) -> Self {
        Self {
            overflow_lines,
            overflowed_verse,
        }
    }
}

// Add Display implementation with conditional colored output
impl std::fmt::Display for OverflowResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[cfg(feature = "colored_output")]
        {
            write!(
                f,
                "{} {:5} {} [{}]",
                "Overflow lines:".black(),
                self.overflow_lines.to_string().red().bold(),
                "Verse:".black(),
                self.overflowed_verse
            )
        }

        #[cfg(not(feature = "colored_output"))]
        {
            write!(
                f,
                "Overflow lines: {}, Verse: {}",
                self.overflow_lines, self.overflowed_verse
            )
        }
    }
}
