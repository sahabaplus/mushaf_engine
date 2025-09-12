use crate::mushaf::Verse;
use super::{ LastVerseResult, OverflowResult };

#[cfg(feature = "colored_output")]
use colored::Colorize;

/// Comprehensive result of a navigation operation in the Quran
///
/// This structure contains the primary verse reached through navigation,
/// as well as optional information about any overflow conditions or boundary
/// verses (last verse of page or sura) encountered during navigation.
#[derive(Debug)]
pub struct NavigationResult {
    /// Reference to the verse reached through navigation
    pub verse: Verse,
    /// Optional information about overflow if navigation exceeded boundaries
    pub overflow: Option<OverflowResult>,
    /// Optional information about the last verse of the page if encountered during navigation
    pub end_of_page: Option<LastVerseResult>,
    /// Optional information about the last verse of the sura if encountered during navigation
    pub end_of_sura: Option<LastVerseResult>,
    /// The actual distance moved during navigation in lines
    pub distance_moved: f32,
    /// The remaining distance that could not be navigated due to boundaries
    pub remaining_distance: f32,
}

impl NavigationResult {
    /// Create a new `NavigationResult` with all possible parameters
    ///
    /// # Arguments
    /// * `verse` - Reference to the verse reached through navigation
    /// * `overflow` - Optional information about overflow if navigation exceeded boundaries
    /// * `last_of_page` - Optional information about the last verse of the page if encountered
    /// * `last_of_sura` - Optional information about the last verse of the sura if encountered
    /// * `distance_moved` - The actual distance moved during navigation in lines
    ///
    /// # Returns
    /// A new `NavigationResult` instance with all specified information
    #[must_use]
    pub fn new(
        verse: Verse,
        overflow: Option<OverflowResult>,
        end_of_page: Option<LastVerseResult>,
        end_of_sura: Option<LastVerseResult>,
        distance_moved: f32
    ) -> Self {
        let remaining_distance = overflow
            .as_ref()
            .map_or(0.0, |v| v.overflowed_verse.lines - v.overflow_lines);
        Self {
            verse,
            remaining_distance: (remaining_distance * 100.0).round() / 100.0,
            overflow,
            end_of_page,
            end_of_sura,
            distance_moved,
        }
    }

    /// Create a new `NavigationResult` for a normal navigation with no boundary conditions
    ///
    /// # Arguments
    /// * `verse` - Reference to the verse reached through navigation
    /// * `distance_moved` - The actual distance moved during navigation in lines
    ///
    /// # Returns
    /// A new `NavigationResult` instance with only the target verse and no boundary information
    #[must_use]
    pub const fn new_normal(verse: Verse, distance_moved: f32) -> Self {
        Self {
            verse,
            overflow: None,
            end_of_page: None,
            end_of_sura: None,
            distance_moved,
            remaining_distance: 0.0,
        }
    }

    /// Create a new `NavigationResult` for a navigation that includes overflow
    ///
    /// # Arguments
    /// * `verse` - Reference to the verse reached through navigation
    /// * `overflow` - Optional information about overflow if navigation exceeded boundaries
    /// * `distance_moved` - The actual distance moved during navigation in lines
    ///
    /// # Returns
    /// A new `NavigationResult` instance with the target verse and overflow information
    #[must_use]
    pub fn new_overflowed(verse: Verse, overflow: OverflowResult, distance_moved: f32) -> Self {
        let remaining_distance = overflow.overflowed_verse.lines - overflow.overflow_lines;
        Self {
            verse,
            remaining_distance: (remaining_distance * 100.0).round() / 100.0,
            overflow: Some(overflow),
            end_of_page: None,
            end_of_sura: None,
            distance_moved,
        }
    }

    /// Check if this navigation result encountered any boundaries
    ///
    /// # Returns
    /// `true` if the navigation hit the last verse of page, sura, or had an overflow
    #[must_use]
    pub const fn has_boundaries(&self) -> bool {
        self.overflow.is_some() || self.end_of_page.is_some() || self.end_of_sura.is_some()
    }

    /// Check if this navigation result encountered an overflow condition
    ///
    /// # Returns
    /// `true` if the navigation exceeded available boundaries
    #[must_use]
    pub const fn has_overflow(&self) -> bool {
        self.overflow.is_some()
    }

    /// Get the total overflow lines if any exist
    ///
    /// # Returns
    /// The number of overflow lines, or 0.0 if no overflow occurred
    #[must_use]
    pub const fn overflow_lines(&self) -> f32 {
        match &self.overflow {
            Some(overflow) => overflow.overflow_lines,
            None => 0.0,
        }
    }

    /// Create a new `NavigationResult` that reached the last verse of a page
    ///
    /// # Arguments
    /// * `verse` - Reference to the verse reached through navigation
    /// * `last_of_page` - Information about the last verse of the page
    /// * `distance_moved` - The actual distance moved during navigation in lines
    ///
    /// # Returns
    /// A new `NavigationResult` instance with the target verse and last-of-page information
    #[must_use]
    pub const fn new_page_boundary(
        verse: Verse,
        last_of_page: LastVerseResult,
        distance_moved: f32
    ) -> Self {
        Self {
            verse,
            overflow: None,
            end_of_page: Some(last_of_page),
            end_of_sura: None,
            distance_moved,
            remaining_distance: 0.0,
        }
    }

    /// Create a new `NavigationResult` that reached the last verse of a sura
    ///
    /// # Arguments
    /// * `verse` - Reference to the verse reached through navigation
    /// * `last_of_sura` - Information about the last verse of the sura
    /// * `distance_moved` - The actual distance moved during navigation in lines
    ///
    /// # Returns
    /// A new `NavigationResult` instance with the target verse and last-of-sura information
    #[must_use]
    pub const fn new_sura_boundary(
        verse: Verse,
        last_of_sura: LastVerseResult,
        distance_moved: f32
    ) -> Self {
        Self {
            verse,
            overflow: None,
            end_of_page: None,
            end_of_sura: Some(last_of_sura),
            distance_moved,
            remaining_distance: 0.0,
        }
    }
}

// Add Display implementation with conditional colored output
impl std::fmt::Display for NavigationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[cfg(feature = "colored_output")]
        {
            // Start with the primary verse information
            writeln!(f, "{:16} {}", "Target verse:".blue().bold(), self.verse)?;
            writeln!(
                f,
                "{:16} {}",
                "Distance Moved:".blue().bold(),
                self.distance_moved.to_string().yellow().bold()
            )?;
            writeln!(
                f,
                "{:16} {}",
                "Remaining Dis:".blue().bold(),
                self.remaining_distance.to_string().yellow().bold()
            )?;

            // Add overflow information if present
            if let Some(overflow) = &self.overflow {
                writeln!(f, "{:16} {}", "Overflow:".yellow().bold(), overflow)?;
            }

            // Add end of page information if present
            if let Some(end_page) = &self.end_of_page {
                writeln!(f, "{:16} {}", "End of page:".cyan().bold(), end_page)?;
            }

            // Add end of sura information if present
            if let Some(end_sura) = &self.end_of_sura {
                writeln!(f, "{:16} {}", "End of sura:".magenta().bold(), end_sura)?;
            }

            Ok(())
        }

        #[cfg(not(feature = "colored_output"))]
        {
            // Start with the primary verse information
            writeln!(f, "Target verse: {}", self.verse)?;

            // Add overflow information if present
            if let Some(overflow) = &self.overflow {
                writeln!(f, "Overflow: {}", overflow)?;
            }

            // Add end of page information if present
            if let Some(end_page) = &self.end_of_page {
                writeln!(f, "Last of page: {}", end_page)?;
            }

            // Add end of sura information if present
            if let Some(end_sura) = &self.end_of_sura {
                writeln!(f, "Last of sura: {}", end_sura)?;
            }

            Ok(())
        }
    }
}
