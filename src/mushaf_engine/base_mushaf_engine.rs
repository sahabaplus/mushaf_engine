use crate::mushaf::{ Mushaf, QuranMetadata, Verse, SuraInfo };
use std::rc::Rc;

use super::{ Direction, IMushafEngine };

/// Basic implementation of the Mushaf navigation engine using King Fahad Mushaf
pub struct BaseMushafEngine {
    pub mushaf: Rc<Mushaf>,
    pub metadata: QuranMetadata,
}

impl BaseMushafEngine {
    /// Create a new BaseMushafEngine with the given Mushaf
    pub fn new(mushaf: Rc<Mushaf>) -> Self {
        let metadata = QuranMetadata::from_mushaf(&mushaf);
        BaseMushafEngine { mushaf, metadata }
    }

    /// Get metadata about a specific Sura
    pub fn get_sura_info(&self, sura_number: u8) -> Option<&SuraInfo> {
        self.metadata.get_sura_info(sura_number)
    }

    /// Find a verse in the mushaf and return its location (Verse, page, index_of_verse)
    fn find_verse(&self, sura_number: u8, verse_number: u16) -> Option<(&Verse, usize, usize)> {
        //     for (verse_idx, v) in page.verses().iter().enumerate() {
        //         if v.sura == sura && v.number == verse {
        //             return Some((page_idx, verse_idx));
        //         }
        //     }
        let sura = self.metadata.get_sura_info(sura_number);
        if let Some(sura) = sura {
            let pages = self.mushaf.pages.as_ref();
            for i in sura.start_page..=sura.end_page {
                let page = &pages[(i - 1) as usize];
                for (i, verse) in page.verses().iter().enumerate() {
                    if verse.sura == sura_number && verse.number == verse_number {
                        return Some((verse, page.number() as usize, i));
                    }
                }
            }
            // for (page_idx, page) in self.mushaf.pages[sura.start_page..=sura.end_page]
            //     .iter()
            //     .enumerate() {
            //     for verse in page.verses().iter() {
            //         if v.sura == sura && v.number == verse {
            //             return Some((page_idx, verse_idx));
            //         }
            //     }
            // }
            None
        } else {
            None
        }
    }

    /// Calculate the lines taken by a verse including any sura headers
    fn calculate_verse_lines(&self, verse: &Verse) -> f32 {
        let mut total_lines = verse.lines;

        // Add lines for sura headers if this is the first verse of a sura
        if verse.number == 1 {
            // All suras except At-Tawbah (9) have bismillah
            if verse.sura != 9 {
                total_lines += 2.0; // Sura title + bismillah
            } else {
                total_lines += 1.0; // Only sura title for At-Tawbah
            }
        }

        total_lines
    }
}

impl IMushafEngine for BaseMushafEngine {
    fn navigate(&self, lines: f32, from_sura: u8, from_verse: u16, direction: Direction) -> &Verse {
        assert!(lines >= 0f32, "lines should be positive or zero");
        // If no lines to navigate, find and return the current verse
        if lines == 0f32 {
            if let Some((verse, _, _)) = self.find_verse(from_sura, from_verse) {
                return verse;
            }
        }

        // Find starting position
        let verse = match self.find_verse(from_sura, from_verse) {
            Some(verse) => verse,
            _ => panic!("Verse Not Found!"),
        };

        let mut remaining_lines = lines as f32;

        let (verse, page, verse_idx) = verse;

        let mushaf = &self.mushaf.pages;
        let mut page_idx = page - 1;
        let mut verse_idx = verse_idx;

        while remaining_lines > 0.0 && page_idx < mushaf.len() {
            let current_verse = mushaf[page_idx].verses()[verse_idx];
            let verse_lines = self.calculate_verse_lines(&current_verse);

            remaining_lines -= verse_lines;

            // If we haven't reached our target yet, move to next verse
            if remaining_lines <= 0.0 {
                break;
            }
            verse_idx += 1;

            // If we've reached the end of current page, move to next page
            if verse_idx >= mushaf[page_idx].verses().len() {
                page_idx += 1;
                verse_idx = 0;

                // Check if we've reached the end of the mushaf
                if page_idx >= mushaf.len() && direction == Direction::Downwards {
                    page_idx = mushaf.len() - 1;
                    verse_idx = mushaf[page_idx].verses().len() - 1;
                    break;
                }
            }

            // Check wither the verse is in different sura
            let next_verse = {
                if page_idx >= mushaf.len() {
                    page_idx = mushaf.len() - 1;
                    Verse::new(115, 1, (0f32, 0), 0f32)
                } else {
                    mushaf[page_idx].verses()[verse_idx]
                }
            };
            if next_verse.sura != current_verse.sura && direction == Direction::Upwards {
                let next_sura = current_verse.sura - 1;
                let next_sura = self.metadata.get_sura_info(next_sura);
                if next_sura.is_none() {
                    // We reached the start of mushaf, we will return the last verse of الفاتحة
                    return &mushaf[0].verses().last().unwrap();
                }
                let next_sura = next_sura.unwrap();
                let new_start_page = next_sura.start_page as usize;
                page_idx = (new_start_page - 1) as usize;

                // find the first verse of next_sura
                'inner: for (i, verse) in mushaf[page_idx].verses().iter().enumerate() {
                    if verse.sura == next_sura.number {
                        verse_idx = i;
                        break 'inner;
                    }
                }
            }
        }

        // Return the verse at our final position
        &self.mushaf.pages[page_idx].verses()[verse_idx]
    }
}
