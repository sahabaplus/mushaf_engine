use crate::{
    mushaf::{SuraInfo, Verse},
    navigation::{
        CalculatingLinesError, Direction, LookupError, NavigationError, NavigationResult,
        NavigationSettings, VersePosition,
    },
};

/// Interface for navigation within a Quran Mushaf
///
/// This trait defines the core navigation functionality for traversing
/// through the Quran. It allows for moving through the text by a specified
/// number of lines in either direction from a given verse.
pub trait IMushafEngine {
    /// Navigate from a verse by a specified number of lines in the given direction
    ///
    /// This method enables precise navigation through the Quran by moving a
    /// specified number of lines forward or backward from a starting verse.
    ///
    /// # Arguments
    /// * `lines` - Number of lines to navigate (can be fractional)
    /// * `from_sura` - Starting Sura number (1-114)
    /// * `from_verse` - Starting verse number within the sura
    /// * `direction` - Direction to navigate (Forward or Backward)
    /// * `settings` - Navigation settings
    ///
    /// # Returns
    /// A reference to the verse at the destination after navigation
    ///
    /// # Errors
    /// * `NavigationError::NegativeLines` if the number of lines is negative
    /// * `NavigationError::InvalidVerse` if the starting verse is invalid
    /// * `NavigationError::OutOfBounds` if the navigation result is out of bounds
    /// * `NavigationError::InvalidSura` if the starting sura is invalid
    /// * `NavigationError::InvalidDirection` if the direction is invalid
    /// * `NavigationError::InvalidLines` if the number of lines is invalid
    fn navigate(
        &self,
        lines: f32,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> Result<NavigationResult, NavigationError>;

    /// Get metadata about a specific Sura
    ///
    /// # Arguments
    /// * `sura_number` - Sura number (1-114)
    ///
    /// # Returns
    /// Information about the specified Sura, or None if not found
    ///
    /// # Errors
    /// * `LookupError::InvalidSura` if the sura number is invalid
    /// * `LookupError::SuraNotFound` if the sura number is not found
    fn get_sura_info(&self, sura_number: u8) -> Result<&SuraInfo, LookupError>;

    /// Calculate the direct distance in lines between start and end verses
    ///
    /// This calculates the simple A→B distance without accounting for cycles.
    /// When bounded navigation produces cycles, callers can use the `cycle_distance`
    /// field from `NavigationResult` to compute total distance:
    ///
    /// ```text
    /// total_distance ≈ (cycles_completed - 1) * cycle_distance + direct_distance
    /// ```
    ///
    /// where `direct_distance` is obtained from this method.
    ///
    /// # Arguments
    /// * `start` - Start verse position `VersePosition`
    /// * `end` - End verse position `VersePosition`
    /// * `direction` - Direction of navigation
    /// * `settings` - Navigation settings
    ///
    /// # Returns
    /// Direct distance in lines between the two verses
    ///
    /// # Errors
    /// * `CalculatingLinesError::WrongBoundary` if the verses are unreachable
    fn calculate_lines(
        &self,
        start: impl Into<VersePosition>,
        end: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> Result<f32, CalculatingLinesError>;

    /// Find the next verse from a given verse in the specified direction.
    /// Respects navigation bounds and iteration limits.
    ///
    /// # Arguments
    /// * `after` - Verse position to start from
    /// * `direction` - Direction to navigate
    /// * `settings` - Navigation settings
    ///
    /// # Returns
    /// The next verse or None if at the end
    fn next_verse(
        &self,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> Option<Verse>;

    /// Find the previous verse from a given verse in the specified direction.
    /// Respects navigation bounds and iteration limits.
    ///
    /// # Arguments
    /// * `from` - Verse position to start from
    /// * `direction` - Direction to navigate
    /// * `settings` - Navigation settings
    ///
    /// # Returns
    /// The previous verse or None if at the start
    fn previous_verse(
        &self,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> Option<Verse>;

    /// Check if a verse position is out of the current navigation bounds.
    ///
    /// Uses the same rules as navigation: the verse must exist in the mushaf,
    /// then is checked against upper/lower bounds and excluding mode. In inclusive
    /// mode, verses on the upper/lower sura edges are interpreted using `direction`,
    /// consistent with the navigator.
    ///
    /// # Arguments
    /// * `verse` - Verse position to check
    /// * `direction` - Navigation direction (bounds checks at upper/lower sura edges are direction-aware)
    /// * `settings` - Navigation settings (bounds and excluding mode)
    ///
    /// # Returns
    /// `true` if the verse is outside the current bounds
    fn is_out_of_bounds(
        &self,
        verse: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> bool;

    /// Get the effective start of the navigation range for the given direction and settings.
    ///
    /// # Arguments
    /// * `direction` - Navigation direction
    /// * `settings` - Navigation settings
    ///
    /// # Returns
    /// The verse position where the range starts
    fn get_start_bound(
        &self,
        direction: Direction,
        settings: NavigationSettings,
    ) -> VersePosition;

    /// Get the effective end of the navigation range for the given direction and settings.
    ///
    /// # Arguments
    /// * `direction` - Navigation direction
    /// * `settings` - Navigation settings
    ///
    /// # Returns
    /// The verse position where the range ends
    fn get_end_bound(
        &self,
        direction: Direction,
        settings: NavigationSettings,
    ) -> VersePosition;

    /// Resolve a (sura, verse) position to the full Verse object in the mushaf.
    ///
    /// # Arguments
    /// * `position` - Verse position to look up
    ///
    /// # Returns
    /// The verse with lines etc., or None if not in the mushaf
    fn find_verse(&self, position: impl Into<VersePosition>) -> Option<Verse>;
}
