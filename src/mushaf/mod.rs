/// Modules
mod mushaf;
mod quran_metadata;
mod verse;
mod page;
/// Re-export
pub use page::Page;
pub use verse::Verse;
pub use quran_metadata::{ QuranMetadata, SuraInfo };
pub use mushaf::Mushaf;