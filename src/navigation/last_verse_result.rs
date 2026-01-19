use crate::mushaf::Verse;

#[cfg(feature = "colored_output")]
use colored::Colorize;

/// Information about the last verse of a page or sura encountered during navigation
///
/// This structure tracks the last verse of either a page or a sura along with
/// the distance of lines that were still available for navigation after reaching this verse.
/// Negative distance of lines indicate an overflow beyond this verse.
#[derive(Debug, Clone)]
pub struct LastVerseResult {
    /// Distance of lines available for navigation (negative indicates overflow)
    pub lines_distance: f32,
    /// Reference to the last verse of the page or sura
    pub last_verse: Verse,
}

impl LastVerseResult {
    /// Create a new `LastVerseResult` with specified parameters
    ///
    /// # Arguments
    /// * `lines_distance` - Distance of lines available for navigation (negative indicates overflow)
    /// * `last_verse` - Reference to the last verse of the page or sura
    ///
    /// # Returns
    /// A new `LastVerseResult` instance
    #[must_use]
    pub const fn new(lines_distance: f32, last_verse: Verse) -> Self {
        Self {
            lines_distance,
            last_verse,
        }
    }
}

// Add Display implementation with conditional colored output
impl std::fmt::Display for LastVerseResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[cfg(feature = "colored_output")]
        {
            let lines_color = if self.lines_distance < 0.0 {
                self.lines_distance.to_string().red().bold()
            } else {
                self.lines_distance.to_string().green().bold()
            };

            write!(
                f,
                "{} {:5} {} [{}]",
                "Lines distance:".black(),
                lines_color,
                "Verse:".black(),
                self.last_verse
            )
        }

        #[cfg(not(feature = "colored_output"))]
        {
            write!(
                f,
                "Lines distance: {}, Verse: {}",
                self.remaining_lines, self.last_verse
            )
        }
    }
}
