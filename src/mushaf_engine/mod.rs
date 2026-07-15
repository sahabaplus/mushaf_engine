//! Engine trait and [`BaseMushafEngine`] implementation.
//!
//! Prefer the crate-root re-exports: [`crate::BaseMushafEngine`], [`crate::IMushafEngine`].

mod i_mushaf_engine;
pub use i_mushaf_engine::IMushafEngine;
pub mod base_mushaf_engine;
pub use base_mushaf_engine::BaseMushafEngine;
mod verse_location;
pub use verse_location::VerseLocation;
