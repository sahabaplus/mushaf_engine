use crate::mushaf::Verse;
use super::direction::Direction;

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
    fn navigate(&self, lines: f32, from_sura: u8, from_verse: u16, direction: Direction) -> &Verse;
}
