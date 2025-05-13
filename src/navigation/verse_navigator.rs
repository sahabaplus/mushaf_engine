use crate::{ mushaf::{ Mushaf, QuranMetadata, Verse } };
use std::rc::Rc;

use super::Direction;

/// A struct responsible for navigating through verses in the Mushaf
pub struct VersesNavigator {
    mushaf: Rc<Mushaf>,
    quran_metadata: Rc<QuranMetadata>,
    current_page_idx: usize,
    current_verse_idx: usize,
}

impl VersesNavigator {
    /// Create a new VersesNavigator
    pub fn new(mushaf: Rc<Mushaf>, metadata: Rc<QuranMetadata>) -> Self {
        VersesNavigator {
            mushaf,
            quran_metadata: metadata,
            current_page_idx: 0,
            current_verse_idx: 0,
        }
    }

    /// Reset the navigator to a specific verse position
    pub fn reset_position(&mut self, sura_number: u8, verse_number: u16) -> Option<&Verse> {
        if let Some((_, page, idx)) = self.find_verse(sura_number, verse_number) {
            self.current_page_idx = page - 1;
            self.current_verse_idx = idx;
            Some(self.current_verse())
        } else {
            None
        }
    }

    /// Get the current verse
    pub fn current_verse(&self) -> &Verse {
        let current_page_verses = self.mushaf.pages[self.current_page_idx].verses();
        &current_page_verses[self.current_verse_idx]
    }

    /// Move to the next verse based on direction
    pub fn next_verse(&mut self, direction: Direction) -> Option<&Verse> {
        match direction {
            Direction::Downwards => self.next_verse_downward(),
            Direction::Upwards => self.next_verse_upward(),
        }
    }

    /// Move to the next verse in downward direction
    pub fn next_verse_downward(&mut self) -> Option<&Verse> {
        let mushaf = &self.mushaf.pages;

        // Try to move to the next verse on the current page
        self.current_verse_idx += 1;

        // If we've reached the end of current page, move to next page
        if self.current_verse_idx >= mushaf[self.current_page_idx].verses().len() {
            self.current_page_idx += 1;
            self.current_verse_idx = 0;

            // Check if we've reached the end of the mushaf
            if self.current_page_idx >= mushaf.len() {
                // Select last verse of quran
                self.current_page_idx = mushaf.len() - 1;
                self.current_verse_idx = mushaf[self.current_page_idx].verses().len() - 1;
            }
        }

        Some(self.current_verse())
    }

    fn move_pre_sura(&mut self, current_sura: u8) -> Option<&Verse> {
        let pre_sura = self.quran_metadata.get_sura_info(current_sura - 1)?;
        let first_page_idx = (pre_sura.start_page - 1) as usize;
        let first_page = &self.mushaf.pages[first_page_idx];
        // find first verse index
        for (idx, verse) in first_page.verses().iter().enumerate() {
            if verse.sura == pre_sura.number {
                self.current_page_idx = first_page_idx;
                self.current_verse_idx = idx;
                return Some(verse);
            }
        }

        unreachable!()
    }

    /// Move to the next verse in upward direction
    pub fn next_verse_upward(&mut self) -> Option<&Verse> {
        let pages = Rc::clone(&self.mushaf.pages);
        let quran_metadata = Rc::clone(&self.quran_metadata);
        let current_verse = *self.current_verse(); // copy verse

        let mut current_verse_idx = self.current_verse_idx + 1;
        let mut current_page_idx = self.current_page_idx;

        // check if the next verse index is valid + in same sura
        if current_verse_idx >= pages[current_page_idx].verses().len() {
            // Move to previous sura
            current_page_idx += 1;
            current_verse_idx = 0;
        }

        if
            current_page_idx >= pages.len() ||
            pages[current_page_idx].verses()[current_verse_idx].sura != current_verse.sura
        {
            return self.move_pre_sura(current_verse.sura);
        }

        // Move next normally
        self.current_verse_idx = current_verse_idx;
        self.current_page_idx = current_page_idx;

        return Some(self.current_verse());
    }

    /// Find a verse in the mushaf and return its location (Verse, page, index_of_verse)
    pub fn find_verse(&self, sura_number: u8, verse_number: u16) -> Option<(&Verse, usize, usize)> {
        let sura = self.quran_metadata.get_sura_info(sura_number);
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
            None
        } else {
            None
        }
    }

    /// Calculate the lines taken by a verse including any sura headers
    pub fn calculate_verse_lines(&self, verse: &Verse) -> f32 {
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

#[cfg(test)]
mod test {
    use std::{ path::PathBuf, rc::Rc };

    use colored::Colorize;

    use crate::{
        king_fahad_mushaf::KingFahadMushaf,
        mushaf::{ Mushaf, QuranMetadata, Verse },
        navigation::{ Direction, VersesNavigator },
    };

    fn get_mushaf() -> (Rc<Mushaf>, Rc<QuranMetadata>) {
        let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data_path.push("data");
        data_path.push("king_fahad_mushaf.json");

        let mushaf = Rc::new(KingFahadMushaf::new(data_path.to_str().unwrap()));
        let metadata = Rc::new(QuranMetadata::from_mushaf(&mushaf));

        (mushaf, metadata)
    }

    #[test]
    fn test_upwards_navigation() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(mushaf, metadata);
        navigator.reset_position(5, 119);

        let verse = navigator.next_verse(Direction::Upwards);
        assert!(verse.is_some());
        println!("{}", verse.unwrap());
        assert_eq!(verse.unwrap().number, 120);
        assert_eq!(verse.unwrap().sura, 5);

        let verse = navigator.next_verse(Direction::Upwards);
        assert!(verse.is_some());
        println!("{}", verse.unwrap());
        assert_eq!(verse.unwrap().number, 1);
        assert_eq!(verse.unwrap().sura, 4);
    }

    #[test]
    fn test_downwards_navigation() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(mushaf, metadata);
        navigator.reset_position(5, 119);

        let verse = navigator.next_verse(Direction::Downwards);
        assert!(verse.is_some());
        println!("{}", verse.unwrap());
        assert_eq!(verse.unwrap().number, 120);
        assert_eq!(verse.unwrap().sura, 5);

        let verse = navigator.next_verse(Direction::Downwards);
        assert!(verse.is_some());
        println!("{}", verse.unwrap());
        assert_eq!(verse.unwrap().number, 1);
        assert_eq!(verse.unwrap().sura, 6);
    }

    #[test]
    fn comprehensive_downwards() {
        let (mushaf, metadata) = get_mushaf();
        let pages = Rc::clone(&mushaf.pages);
        let mut navigator = VersesNavigator::new(mushaf, metadata);

        for page in pages.iter() {
            for verse in page.verses() {
                assert_eq!(*verse, *navigator.current_verse());

                navigator.next_verse_downward();
            }
        }
    }
    #[test]
    fn comprehensive_upwards() {
        let (mushaf, metadata) = get_mushaf();
        let pages = Rc::clone(&mushaf.pages);
        let mut navigator = VersesNavigator::new(mushaf, metadata.clone());

        let mut current_verse = pages[0].verses()[0];
        navigator.reset_position(114, 1);
        for i in 1..=114 as u8 {
            let sura_number = 114 - i + 1;
            let sura = metadata.get_sura_info(sura_number).expect("Invalid sura number");
            for page_number in sura.start_page..=sura.end_page {
                let page = &pages[(page_number - 1) as usize];
                for verse in page.verses() {
                    if verse.sura != sura_number {
                        continue;
                    }
                    assert_eq!(*verse, *navigator.current_verse());
                    navigator.next_verse_upward();
                }
            }
        }
    }
}
