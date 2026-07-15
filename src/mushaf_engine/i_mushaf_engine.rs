use crate::{
    mushaf::{SuraInfo, Verse},
    navigation::{
        CalculatingLinesError, Direction, LookupError, NavigationError, NavigationResult,
        NavigationSettings, VersePosition,
    },
};

use super::verse_location::VerseLocation;

/// Line-based navigation within a Quran Mushaf.
///
/// Implemented by [`super::BaseMushafEngine`]. Prefer importing
/// [`crate::IMushafEngine`] from the crate root.
pub trait IMushafEngine {
    /// Navigate by lines stepping forward through verses via [`Self::next_verse`].
    ///
    /// # Arguments
    /// * `lines` - Lines to move (may be fractional; must be ≥ 0)
    /// * `from` - Starting verse position
    /// * `direction` - Reading order: [`Direction::Downwards`] (1→114) or [`Direction::Upwards`] (114→1)
    /// * `settings` - Bounds, cycling, and sura-header handling
    ///
    /// # Errors
    /// * [`NavigationError::NegativeLines`] if `lines` is negative
    /// * [`NavigationError::InvalidVerse`] if the starting verse is invalid
    /// * [`NavigationError::OutOfBounds`] if the result is outside configured bounds
    fn navigate(
        &self,
        lines: f32,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> Result<NavigationResult, NavigationError>;

    /// Navigate by lines stepping backward through verses via [`Self::previous_verse`].
    ///
    /// Line counting and cycle tracking match [`Self::navigate`]; only the step
    /// function changes (`previous_verse` instead of `next_verse`).
    ///
    /// This is **not** the same as calling [`Self::navigate`] with the opposite
    /// [`Direction`]. See `docs/reverse-navigate.md` in the repository.
    ///
    /// # Errors
    /// Same as [`Self::navigate`].
    fn reverse_navigate(
        &self,
        lines: f32,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> Result<NavigationResult, NavigationError>;

    /// Get metadata about a specific Sura
    ///
    /// # Errors
    /// * `LookupError::InvalidSura` if the sura number is invalid
    /// * `LookupError::SuraNotFound` if the sura number is not found
    fn get_sura_info(&self, sura_number: u8) -> Result<&SuraInfo, LookupError>;

    /// Calculate the direct distance in lines between start and end verses
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
    fn next_verse(
        &self,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> Option<Verse>;

    /// Find the previous verse from a given verse in the specified direction.
    fn previous_verse(
        &self,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> Option<Verse>;

    /// Check if a verse position is out of the current navigation bounds.
    fn is_out_of_bounds(
        &self,
        verse: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> bool;

    /// Get the effective start of the navigation range for the given direction and settings.
    fn get_start_bound(&self, direction: Direction, settings: NavigationSettings) -> VersePosition;

    /// Get the effective end of the navigation range for the given direction and settings.
    fn get_end_bound(&self, direction: Direction, settings: NavigationSettings) -> VersePosition;

    /// Resolve a (sura, verse) position to the full Verse object in the mushaf.
    fn find_verse(&self, position: impl Into<VersePosition>) -> Option<Verse>;

    /// Find a verse and return its location (verse, page, index on page).
    fn find_verse_location(&self, position: impl Into<VersePosition>) -> Option<VerseLocation>;

    /// Resolve a (sura, verse) pair directly to a Verse object.
    fn resolve_position(&self, sura: u8, verse: u16) -> Option<Verse>;

    /// Calculate the lines taken by a verse including any sura headers.
    fn calculate_verse_lines(&self, verse: &Verse, settings: NavigationSettings) -> f32;

    /// Check if navigation direction is wrong for given start and end positions.
    fn is_wrong_direction(
        &self,
        start: impl Into<VersePosition>,
        end: impl Into<VersePosition>,
        direction: Direction,
    ) -> bool;
}
