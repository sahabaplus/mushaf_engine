use std::default;

use crate::mushaf::Verse;

mod base_mushaf_engine;

/// Direction for navigating through the Quran
///
/// # Variants
///
/// * `Downwards` - Navigate from Sura Al-Fatiha (1) towards Sura An-Nas (114)
/// * `Upwards` - Navigate from Sura An-Nas (114) towards Sura Al-Fatiha (1)
#[derive(Debug, Default, Clone, Copy)]
pub enum Direction {
    #[default]
    Downwards,
    Upwards,
}

/// Interface for Mushaf navigation engine implementations
///
/// This trait defines the contract for navigating through Quranic verses
/// based on a specified number of lines, starting position, and direction.
pub trait IMushafEngine {
    /// Navigate through the Quran based on line count
    ///
    /// # Arguments
    ///
    /// * `lines` - Number of lines to navigate
    /// * `from_sura` - Starting Sura number (1-114)
    /// * `from_verse` - Starting verse number within the Sura
    /// * `direction` - Direction of navigation (Downwards or Upwards)
    ///
    /// # Returns
    ///
    /// The `Verse` at the target position after navigation
    fn navigate(&self, lines: f32, from_sura: u8, from_verse: u16, direction: Direction) -> Verse;
}
