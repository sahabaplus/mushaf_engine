//! A verse resolved in the mushaf together with its page placement.

use crate::mushaf::Verse;

/// Location of a verse inside the mushaf layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VerseLocation {
    /// The verse instance from the mushaf layout.
    pub verse: Verse,
    /// 1-based page number in this mushaf.
    pub page_number: u16,
    /// 0-based index of this verse in that page's `verses()` array.
    pub verse_index_on_page: u8,
}
