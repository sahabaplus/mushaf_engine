use std::{ error::Error, fmt::Display };

use crate::{
    mushaf::{ SuraInfo, Verse },
    navigation::{
        CalculatingLinesError,
        Direction,
        LookupError,
        NavigationError,
        NavigationResult,
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
        from_sura: u8,
        from_verse: u16,
        direction: Direction
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

    /// Calculate the lines between start and end verses
    ///
    /// # Arguments
    /// * `start_sura` - Start sura number
    /// * `start_verse` - Start verse number
    /// * `end_sura` - End sura number
    /// * `end_verse` - End verse number
    /// * `direction` - Direction of navigation
    ///
    /// # Returns
    /// Number of lines between the two verses
    /// 
    /// # Errors
    /// * `CalculatingLinesError::WrongBoundary` if the start and end verses are not in the same sura
    fn calculate_lines(
        &self,
        start_sura: u8,
        start_verse: u16,
        end_sura: u8,
        end_verse: u16,
        direction: Direction
    ) -> Result<f32, CalculatingLinesError>;

    /// Find the next verse from a given verse in the specified direction
    ///
    /// # Arguments
    /// * `sura_number` - Current sura number
    /// * `verse_number` - Current verse number
    /// * `direction` - Direction to navigate
    ///
    /// # Returns
    /// The next verse or None if at the end
    fn next_verse(&self, sura_number: u8, verse_number: u16, direction: Direction) -> Option<Verse>;
}
