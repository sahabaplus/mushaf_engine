use crate::mushaf::{ Mushaf, Verse };
use std::rc::Rc;

use super::{ Direction, IMushafEngine };

/// Basic implementation of the Mushaf navigation engine using King Fahad Mushaf
pub struct BaseMushafEngine {
    mushaf: Rc<Mushaf>,
}

impl BaseMushafEngine {
    /// Create a new BaseMushafEngine with the given Mushaf
    pub fn new(mushaf: Rc<Mushaf>) -> Self {
        BaseMushafEngine { mushaf }
    }

    /// Find a verse in the mushaf and return its location (page_idx, verse_idx)
    fn find_verse(&self, sura: u8, verse: u16) -> Option<(usize, usize)> {
        for (page_idx, page) in self.mushaf.pages.iter().enumerate() {
            for (verse_idx, v) in page.verses().iter().enumerate() {
                if v.sura == sura && v.number == verse {
                    return Some((page_idx, verse_idx));
                }
            }
        }
        None
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
    fn navigate(&self, lines: f32, from_sura: u8, from_verse: u16, direction: Direction) -> Verse {
        assert!(lines >= 0f32, "lines should be positive or zero");
        // If no lines to navigate, find and return the current verse
        if lines == 0f32 {
            if let Some((page_idx, verse_idx)) = self.find_verse(from_sura, from_verse) {
                return self.mushaf.pages[page_idx].verses()[verse_idx];
            }
        }

        // Find starting position
        let (mut page_idx, mut verse_idx) = match self.find_verse(from_sura, from_verse) {
            Some(position) => position,
            None =>
                match direction {
                    // If verse not found, start at beginning or end based on direction
                    Direction::Downwards => (0, 0),
                    Direction::Upwards => {
                        let last_page_idx = self.mushaf.pages.len() - 1;
                        let last_verse_idx = self.mushaf.pages[last_page_idx].verses().len() - 1;
                        (last_page_idx, last_verse_idx)
                    }
                }
        };

        let mut remaining_lines = lines as f32;

        match direction {
            Direction::Downwards => {
                // Navigate downwards through pages and verses
                while remaining_lines > 0.0 && page_idx < self.mushaf.pages.len() {
                    let current_verse = self.mushaf.pages[page_idx].verses()[verse_idx];
                    let verse_lines = self.calculate_verse_lines(&current_verse);

                    remaining_lines -= verse_lines;

                    // If we haven't reached our target yet, move to next verse
                    if remaining_lines > 0.0 {
                        verse_idx += 1;

                        // If we've reached the end of current page, move to next page
                        if verse_idx >= self.mushaf.pages[page_idx].verses().len() {
                            page_idx += 1;
                            verse_idx = 0;

                            // Check if we've reached the end of the mushaf
                            if page_idx >= self.mushaf.pages.len() {
                                page_idx = self.mushaf.pages.len() - 1;
                                verse_idx = self.mushaf.pages[page_idx].verses().len() - 1;
                                break;
                            }
                        }
                    }
                }
            }
            Direction::Upwards => {
                // Navigate upwards through pages and verses
                while remaining_lines > 0.0 && (page_idx > 0 || verse_idx > 0) {
                    let current_verse = self.mushaf.pages[page_idx].verses()[verse_idx];
                    let verse_lines = self.calculate_verse_lines(&current_verse);

                    remaining_lines -= verse_lines;

                    // If we haven't reached our target yet, move to previous verse
                    if remaining_lines > 0.0 {
                        // If we're at the first verse of the page
                        if verse_idx == 0 {
                            // Move to previous page if possible
                            if page_idx > 0 {
                                page_idx -= 1;
                                verse_idx = self.mushaf.pages[page_idx].verses().len() - 1;
                            } else {
                                // We're at the first verse of the first page
                                break;
                            }
                        } else {
                            // Move to previous verse on current page
                            verse_idx -= 1;
                        }
                    }
                }
            }
        }

        // Return the verse at our final position
        self.mushaf.pages[page_idx].verses()[verse_idx]
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::*;
    use crate::king_fahad_mushaf::KingFahadMushaf;

    fn setup_engine() -> BaseMushafEngine {
        let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data_path.push("data");
        data_path.push("king_fahad_mushaf.json");

        let mushaf = Rc::new(KingFahadMushaf::new(data_path.to_str().unwrap()));
        BaseMushafEngine::new(mushaf)
    }

    #[test]
    fn navigate_zero_lines() {
        let engine = setup_engine();

        // First verse of Al-Fatiha
        let result = engine.navigate(0f32, 1, 1, Direction::Downwards);
        assert_eq!(result.sura, 1);
        assert_eq!(result.number, 1);
    }

    #[test]
    fn navigate_downwards() {
        let engine = setup_engine();

        // Navigate from Al-Fatiha verse 1 downwards by 15 lines (one page)
        let result = engine.navigate(15f32, 1, 1, Direction::Downwards);

        // Should be on page 2
        let page2 = engine.mushaf.get_page(2).unwrap();
        let on_page2 = page2
            .verses()
            .iter()
            .any(|v| v.sura == result.sura && v.number == result.number);

        assert!(on_page2, "Result verse should be on page 2");
    }

    #[test]
    fn navigate_upwards() {
        let engine = setup_engine();

        // Find a verse on page 3
        let page3 = engine.mushaf.get_page(3).unwrap();
        let first_verse = page3.verses()[0];

        // Navigate upwards by 15 lines
        let result = engine.navigate(
            15f32,
            first_verse.sura,
            first_verse.number,
            Direction::Upwards
        );

        // Should be on page 2
        let page2 = engine.mushaf.get_page(2).unwrap();
        let on_page2 = page2
            .verses()
            .iter()
            .any(|v| v.sura == result.sura && v.number == result.number);

        assert!(on_page2, "Result verse should be on page 2");
    }

    #[test]
    fn boundaries() {
        let engine = setup_engine();

        // Test navigating past the end of the Quran
        let last_page_idx = engine.mushaf.pages.len() - 1;
        let last_page = &engine.mushaf.pages[last_page_idx];
        let last_verse = last_page.verses()[last_page.verses().len() - 1];

        let result = engine.navigate(
            1000f32,
            last_verse.sura,
            last_verse.number,
            Direction::Downwards
        );
        assert_eq!(result.sura, last_verse.sura);
        assert_eq!(result.number, last_verse.number);

        // Test navigating past the beginning of the Quran
        let first_verse = engine.mushaf.pages[0].verses()[0];
        let result = engine.navigate(
            1000f32,
            first_verse.sura,
            first_verse.number,
            Direction::Upwards
        );
        assert_eq!(result.sura, first_verse.sura);
        assert_eq!(result.number, first_verse.number);
    }

    #[test]
    fn sura_transitions() {
        let engine = setup_engine();

        // Test transitioning between suras
        let mut sura2_verse1_location = None;

        // Find Sura 2, Verse 1
        for (page_idx, page) in engine.mushaf.pages.iter().enumerate() {
            for (verse_idx, verse) in page.verses().iter().enumerate() {
                if verse.sura == 2 && verse.number == 1 {
                    sura2_verse1_location = Some((page_idx, verse_idx));
                    break;
                }
            }
            if sura2_verse1_location.is_some() {
                break;
            }
        }

        if let Some((page_idx, verse_idx)) = sura2_verse1_location {
            let sura2_verse1 = engine.mushaf.pages[page_idx].verses()[verse_idx];

            // Navigate upwards to transition from Al-Baqarah to Al-Fatiha
            let result = engine.navigate(
                5f32,
                sura2_verse1.sura,
                sura2_verse1.number,
                Direction::Upwards
            );
            assert_eq!(result.sura, 1, "Should transition from Sura 2 to Sura 1");
        } else {
            panic!("Could not find Sura 2, Verse 1 for testing");
        }
    }
}
