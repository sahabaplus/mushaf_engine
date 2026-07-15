//! Mushaf model: pages, verses, and sura metadata.

#[allow(clippy::module_inception)]
mod mushaf;
mod page;
mod quran_metadata;
mod verse;

pub use mushaf::Mushaf;
pub use page::Page;
pub use quran_metadata::{QuranMetadata, SuraInfo};
pub use verse::Verse;
