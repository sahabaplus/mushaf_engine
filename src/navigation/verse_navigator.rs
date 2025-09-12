use crate::{
    mushaf::{ Mushaf, QuranMetadata, Verse },
    navigation::{ LookupError, NavigationSettings },
};
use std::rc::Rc;

use super::Direction;

/// A struct responsible for navigating through verses in the Mushaf
pub struct VersesNavigator {
    mushaf: Rc<Mushaf>,
    quran_metadata: Rc<QuranMetadata>,
    current_page_idx: usize,
    current_verse_idx: usize,
    settings: NavigationSettings,
    direction: Direction,
}

impl VersesNavigator {
    /// Create a new `VersesNavigator`
    #[must_use]
    pub const fn new(
        mushaf: Rc<Mushaf>,
        metadata: Rc<QuranMetadata>,
        settings: NavigationSettings,
        direction: Direction
    ) -> Self {
        Self {
            mushaf,
            quran_metadata: metadata,
            current_page_idx: 0,
            current_verse_idx: 0,
            settings,
            direction,
        }
    }

    #[must_use]
    pub fn builder(mushaf: Rc<Mushaf>, metadata: Rc<QuranMetadata>) -> Self {
        Self::new(mushaf, metadata, Default::default(), Default::default())
    }

    #[must_use]
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
    #[must_use]
    pub fn settings(mut self, settings: NavigationSettings) -> Self {
        self.settings = settings;
        self
    }
    #[must_use]
    pub fn ignore_sura_header(mut self, ignore_sura_header: bool) -> Self {
        self.settings.ignore_sura_header = ignore_sura_header;
        self
    }

    /// Reset the navigator to a specific verse position
    ///
    /// # Errors
    /// * `LookupError::InvalidVerse` if the verse number is invalid
    pub fn reset_position(
        &mut self,
        sura_number: u8,
        verse_number: u16
    ) -> Result<&Verse, LookupError> {
        let (_, page, idx) = self.find_verse(sura_number, verse_number)?;
        self.current_page_idx = (page as usize) - 1;
        self.current_verse_idx = idx as usize;
        Ok(self.current_verse())
    }

    /// Get the current verse
    #[must_use]
    pub fn current_verse(&self) -> &Verse {
        let current_page_verses = self.mushaf.pages[self.current_page_idx].verses();
        &current_page_verses[self.current_verse_idx]
    }

    /// Move to the next verse based on direction
    pub fn next_verse(&mut self) -> Option<&Verse> {
        match self.direction {
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
                self.current_page_idx -= 1; // Back to last page.
                // Select last verse of quran
                // self.current_page_idx = mushaf.len() - 1;
                // self.current_verse_idx = mushaf[self.current_page_idx].verses().len() - 1;
                return None;
            }
        }

        Some(self.current_verse())
    }

    fn move_pre_sura(&mut self, current_sura: u8) -> Result<&Verse, LookupError> {
        let pre_sura = self.quran_metadata.get_sura_info(current_sura - 1)?;
        let first_page_idx = (pre_sura.start_page - 1) as usize;
        let first_page = &self.mushaf.pages[first_page_idx];
        // find first verse index
        for (idx, verse) in first_page.verses().iter().enumerate() {
            if verse.sura == pre_sura.number {
                self.current_page_idx = first_page_idx;
                self.current_verse_idx = idx;
                return Ok(verse);
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
            match self.move_pre_sura(current_verse.sura) {
                Ok(verse) => {
                    return Some(verse);
                }
                Err(e) => {
                    return None;
                }
            }
        }

        // Move next normally
        self.current_verse_idx = current_verse_idx;
        self.current_page_idx = current_page_idx;

        Some(self.current_verse())
    }

    /// Find a verse in the mushaf and return its location (Verse, `page`, `index_of_verse`)
    ///
    /// # Errors
    /// * `LookupError::InvalidVerse` if the verse number is invalid
    /// * `LookupError::VerseNotFound` if the verse is not found
    ///
    /// # Panics
    /// * `expect` if `i` cannot be converted to a `u8`
    pub fn find_verse(
        &self,
        sura_number: u8,
        verse_number: u16
    ) -> Result<(&Verse, u16, u8), LookupError> {
        let sura = self.quran_metadata.get_sura_info(sura_number)?;

        if verse_number > sura.total_verses || verse_number < 1 {
            return Err(LookupError::InvalidVerse(verse_number));
        }

        let pages = self.mushaf.pages.as_ref();
        for i in sura.start_page..=sura.end_page {
            let page = &pages[(i - 1) as usize];
            for (i, verse) in page.verses().iter().enumerate() {
                if verse.sura == sura_number && verse.number == verse_number {
                    let i = u8::try_from(i).expect("expect `i` to be less than `u8::MAX`");
                    return Ok((verse, page.number(), i));
                }
            }
        }
        Err(LookupError::VerseNotFound(verse_number))
    }

    /// Calculate the lines taken by a verse including any sura headers
    #[must_use]
    #[inline]
    pub fn calculate_verse_lines(&self, verse: &Verse) -> f32 {
        let mut total_lines = verse.lines;
        if self.settings.ignore_sura_header {
            return total_lines;
        }

        // Add lines for sura headers if this is the first verse of a sura
        if verse.number == 1 {
            // All suras except At-Tawbah (9) have bismillah
            if verse.sura == 9 {
                total_lines += 1.0; // Only sura title for At-Tawbah
            } else {
                total_lines += 2.0; // Sura title + bismillah
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
        king_fahad_mushaf::{ JsonVerse, KingFahadMushaf },
        mushaf::{ Mushaf, QuranMetadata, Verse },
        navigation::{ Direction, VersesNavigator },
    };

    fn get_mushaf() -> (Rc<Mushaf>, Rc<QuranMetadata>) {
        let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data_path.push("data");
        data_path.push("king_fahad_mushaf.json");

        let mushaf = Rc::new({
            let path = data_path.to_str().expect("expect `data_path` to be a valid string");
            // Load from provided JSON path
            let file_content = std::fs
                ::read_to_string(path)
                .expect("Failed to read mushaf data file");

            let pages: Vec<Vec<JsonVerse>> = serde_json
                ::from_str(&file_content)
                .expect("Failed to parse mushaf JSON data");

            KingFahadMushaf::create_mushaf_from_pages(pages)
        });
        let metadata = Rc::new(QuranMetadata::from_mushaf(&mushaf));

        (mushaf, metadata)
    }

    #[test]
    fn test_calculate_verse_with_headers_and_without_headers() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        ).ignore_sura_header(false);
        // Sura (9 - At-Tawbah) does not have a bismillah
        navigator.reset_position(9, 1);
        let verse = navigator.current_verse();
        let lines = navigator.calculate_verse_lines(verse);
        assert!((lines - (verse.lines + 1.0)).abs() < f32::EPSILON);

        navigator.reset_position(9, 1);
        let mut navigator = navigator.ignore_sura_header(true);
        let verse = navigator.current_verse();
        let lines = navigator.calculate_verse_lines(verse);
        assert_eq!(lines, verse.lines);

        // Loop through all first verses of suras
        navigator = navigator.ignore_sura_header(false);
        for sura in 1..=114 {
            let header_lines = if sura == 9 { 1.0 } else { 2.0 };
            navigator.reset_position(sura, 1);
            let verse = navigator.current_verse();
            let lines = navigator.calculate_verse_lines(verse);
            assert!((lines - (verse.lines + header_lines)).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_upwards_navigation() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        );
        navigator.reset_position(5, 119);

        let verse = navigator.next_verse();
        let verse = verse.expect("expect `verse` not to be None");
        println!("{verse}");
        assert_eq!(verse.number, 120);
        assert_eq!(verse.sura, 5);

        let mut navigator = navigator.direction(Direction::Upwards);
        let verse = navigator.next_verse();
        let verse = verse.expect("expect `verse` not to be None");
        println!("{verse}");
        assert_eq!(verse.number, 1);
        assert_eq!(verse.sura, 4);
    }

    #[test]
    fn test_downwards_navigation() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        );
        navigator.reset_position(5, 119);

        let verse = navigator.next_verse();
        let verse = verse.expect("expect `verse` not to be None");
        println!("{verse}");
        assert_eq!(verse.number, 120);
        assert_eq!(verse.sura, 5);

        let mut navigator = navigator.direction(Direction::Downwards);
        let verse = navigator.next_verse();
        let verse = verse.expect("expect `verse` not to be None");
        println!("{verse}");
        assert_eq!(verse.number, 1);
        assert_eq!(verse.sura, 6);
    }

    #[test]
    fn comprehensive_downwards() {
        let (mushaf, metadata) = get_mushaf();
        let pages = Rc::clone(&mushaf.pages);
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        );

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
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata.clone(),
            Default::default(),
            Default::default()
        );

        navigator.reset_position(114, 1);
        for i in 1..=114_u8 {
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

    #[test]
    fn per_sura() {
        let (mushaf, metadata) = get_mushaf();
        let pages = Rc::clone(&mushaf.pages);
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        );
        navigator.reset_position(114, 6);

        let pre = navigator.move_pre_sura(navigator.current_verse().sura);
        let pre = pre.expect("expect `pre` not to be Err");
        assert_eq!(pre.sura, 113);

        for i in 1..=113 {
            let sura = 115 - i;
            let pre = navigator.move_pre_sura(sura);
            let pre = pre.expect("expect `pre` not to be Err");
            assert_eq!(pre.sura, sura - 1);
        }
    }
}
