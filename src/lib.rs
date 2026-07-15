//! Line-based Quran Mushaf navigation engine for the King Fahad Quran Printing
//! Complex Mushaf (15 lines per page).
//!
//! # Quick start
//!
//! ```no_run
//! use mushaf_engine::{
//!     BaseMushafEngine, Direction, IMushafEngine, NavigationSettings, VersePosition,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let engine = BaseMushafEngine::king_fahad()?;
//!
//! let result = engine.navigate(
//!     15.0,
//!     VersePosition::new(5, 3),
//!     Direction::Downwards,
//!     NavigationSettings::builder(),
//! )?;
//!
//! println!("{result}");
//! # Ok(())
//! # }
//! ```
//!
//! # Reverse navigation
//!
//! [`IMushafEngine::reverse_navigate`] steps with [`IMushafEngine::previous_verse`]
//! instead of [`IMushafEngine::next_verse`]. That is **not** the same as flipping
//! [`Direction`] — see the [reverse navigation guide](https://github.com/sahabaplus/mushaf_engine/blob/main/docs/reverse-navigate.md)
//! in the repository (`docs/reverse-navigate.md`).
//!
//! # Modules
//!
//! - [`mushaf`] — `Mushaf`, `Page`, `Verse`, sura metadata
//! - [`mushaf_engine`] — [`BaseMushafEngine`] and [`IMushafEngine`]
//! - [`navigation`] — settings, bounds, results, [`VersePosition`], [`Direction`]
//! - [`king_fahad_mushaf`] — JSON loader (feature `king_fahad_mushaf`, on by default)

pub mod mushaf;
pub mod mushaf_engine;
pub mod navigation;

#[cfg(feature = "king_fahad_mushaf")]
pub mod king_fahad_mushaf;

// Crate-root re-exports for an ergonomic public API (moyasar-rs style).
pub use mushaf::{Mushaf, Page, QuranMetadata, SuraInfo, Verse};
pub use mushaf_engine::{BaseMushafEngine, IMushafEngine, VerseLocation};
pub use navigation::{
    CalculatingLinesError, CycleInfo, Direction, LastVerseResult, LookupError, NavigationBounds,
    NavigationError, NavigationResult, NavigationSettings, OverflowResult, VersePosition,
    VersesNavigator,
};

#[cfg(feature = "king_fahad_mushaf")]
pub use king_fahad_mushaf::{JsonVerse, KingFahadMushaf};
