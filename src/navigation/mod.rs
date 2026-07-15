//! Navigation types: direction, bounds, settings, and results.
//!
//! Prefer the crate-root re-exports (`Direction`, `NavigationSettings`, `VersePosition`, …).

mod direction;
mod error;
mod last_verse_result;
mod navigation_bounds;
mod navigation_result;
mod navigation_settings;
mod overflow_result;
mod verse_navigator;
mod verse_position;

pub use direction::Direction;
pub use error::{CalculatingLinesError, LookupError, NavigationError};
pub use last_verse_result::LastVerseResult;
pub use navigation_bounds::NavigationBounds;
pub use navigation_result::{CycleInfo, NavigationResult};
pub use navigation_settings::NavigationSettings;
pub use overflow_result::OverflowResult;
pub use verse_navigator::VersesNavigator;
pub use verse_position::VersePosition;
