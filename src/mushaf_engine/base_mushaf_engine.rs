use crate::{
    mushaf::{ Mushaf, QuranMetadata, SuraInfo, Verse },
    navigation::{
        CalculatingLinesError,
        Direction,
        LastVerseResult,
        LookupError,
        NavigationError,
        NavigationResult,
        OverflowResult,
        VersesNavigator,
    },
};
use std::rc::Rc;

use super::{ IMushafEngine };

/// Basic implementation of the Mushaf navigation engine using King Fahad Mushaf
pub struct BaseMushafEngine {
    pub mushaf: Rc<Mushaf>,
    pub quran_metadata: Rc<QuranMetadata>,
    pub navigator: VersesNavigator,
}

impl BaseMushafEngine {
    /// Create a new `BaseMushafEngine` with the given Mushaf
    #[must_use]
    pub fn new(mushaf: Rc<Mushaf>) -> Self {
        let quran_metadata = Rc::new(QuranMetadata::from_mushaf(&mushaf));
        let navigator = VersesNavigator::new(mushaf.clone(), quran_metadata.clone());
        Self { mushaf, quran_metadata, navigator }
    }

    /// Get metadata about a specific Sura
    ///
    /// # Errors
    ///
    /// Returns `LookupError` if the sura number is invalid or not found
    pub fn get_sura_info(&self, sura_number: u8) -> Result<&SuraInfo, LookupError> {
        self.quran_metadata.get_sura_info(sura_number)
    }

    const fn is_wrong_direction(start_sura: u8, end_sura: u8, direction: Direction) -> bool {
        match direction {
            Direction::Downwards => start_sura > end_sura,
            Direction::Upwards => start_sura < end_sura,
        }
    }

    fn prefer_last_of_sura(
        &self,
        last_of_sura: Option<LastVerseResult>,
        current_verse: &Verse,
        direction: Direction
    ) -> LastVerseResult {
        if let Some(last_result) = &last_of_sura {
            if current_verse.sura == last_result.last_verse.sura {
                return last_result.clone();
            }
        }

        let sura = self.quran_metadata
            .get_sura_info(current_verse.sura)
            .expect("Sura should exist for current verse");
        let last_verse = self.navigator
            .find_verse(sura.number, sura.total_verses)
            .expect("Last verse of sura should exist").0;

        let lines_distance =
            self
                .calculate_lines(
                    current_verse.sura,
                    current_verse.number,
                    last_verse.sura,
                    last_verse.number,
                    direction
                )
                .expect("Lines calculation should succeed") - current_verse.lines;

        if let Some(existing_last) = last_of_sura {
            if existing_last.lines_distance.abs() + 3.0 < lines_distance.abs() {
                return existing_last;
            }
        }

        LastVerseResult::new(lines_distance, *last_verse)
    }

    fn prefer_last_of_page(
        &self,
        last_of_page: Option<LastVerseResult>,
        current_verse: &Verse,
        direction: Direction
    ) -> Option<LastVerseResult> {
        if let Some(existing_last) = &last_of_page {
            if *current_verse == existing_last.last_verse {
                return last_of_page;
            }
        }

        let (_, page, _idx) = self.navigator
            .find_verse(current_verse.sura, current_verse.number)
            .ok()?;
        let new_last_of_page = self.mushaf.get_page(page)?.verses().last()?;

        if Self::is_wrong_direction(current_verse.sura, new_last_of_page.sura, direction) {
            return last_of_page;
        }

        let lines_distance = self
            .calculate_lines(
                current_verse.sura,
                current_verse.number,
                new_last_of_page.sura,
                new_last_of_page.number,
                direction
            )
            .ok()?;

        if let Some(existing_last) = last_of_page {
            if existing_last.lines_distance.abs() + 1.0 < lines_distance.abs() {
                return Some(existing_last);
            }
        }

        Some(LastVerseResult::new(lines_distance, *new_last_of_page))
    }
}

impl IMushafEngine for BaseMushafEngine {
    fn navigate(
        &self,
        lines: f32,
        from_sura: u8,
        from_verse: u16,
        direction: Direction
    ) -> Result<NavigationResult, NavigationError> {
        if lines < 0.0 {
            return Err(NavigationError::NegativeLines);
        }

        // If no lines to navigate, find and return the current verse
        if lines == 0.0 {
            if let Ok((verse, _page, _idx)) = self.navigator.find_verse(from_sura, from_verse) {
                return Ok(NavigationResult::new_normal(*verse, 0.0));
            }
            return Err(NavigationError::InvalidVerse);
        }

        // Find starting position
        let Ok((verse, _page, _verse_idx)) = self.navigator.find_verse(from_sura, from_verse) else {
            return Err(NavigationError::InvalidVerse);
        };

        let mut remaining_lines = lines;

        let mut navigator = VersesNavigator::new(self.mushaf.clone(), self.quran_metadata.clone());
        navigator.reset_position(verse.sura, verse.number);

        let mut overflow: Option<OverflowResult> = None;
        let mut previous_verse = *navigator.current_verse();

        let mut last_of_page: Option<LastVerseResult> = None;
        let mut last_of_sura: Option<LastVerseResult> = None;

        loop {
            let current_verse = navigator.current_verse();
            let current_sura_info = self.quran_metadata
                .get_sura_info(current_verse.sura)
                .expect("Current verse sura should exist");
            let verse_lines = navigator.calculate_verse_lines(current_verse);
            let diff = ((remaining_lines - verse_lines) * 100.0).round() / 100.0;

            if current_verse.is_last_of_page() {
                last_of_page = Some(LastVerseResult::new(-diff, *current_verse));
            }

            if current_sura_info.total_verses == current_verse.number {
                last_of_sura = Some(LastVerseResult::new(-diff, *current_verse));
            }

            if diff < 0.0 {
                overflow = Some(OverflowResult::new(-diff, *current_verse));
                break;
            }

            remaining_lines = diff;
            previous_verse = *current_verse;

            if remaining_lines > 0.0 {
                navigator.next_verse(direction);
            } else {
                break;
            }
        }

        let last_of_sura = self.prefer_last_of_sura(last_of_sura, &previous_verse, direction);
        let last_of_page = self.prefer_last_of_page(last_of_page, &previous_verse, direction);

        Ok(
            NavigationResult::new(
                previous_verse,
                overflow,
                last_of_page,
                Some(last_of_sura),
                lines - remaining_lines
            )
        )
    }

    fn get_sura_info(&self, sura_number: u8) -> Result<&SuraInfo, LookupError> {
        self.quran_metadata.get_sura_info(sura_number)
    }

    fn calculate_lines(
        &self,
        start_sura: u8,
        start_verse: u16,
        end_sura: u8,
        end_verse: u16,
        direction: Direction
    ) -> Result<f32, CalculatingLinesError> {
        let is_wrong_direction = Self::is_wrong_direction(start_sura, end_sura, direction);
        if is_wrong_direction || (start_sura == end_sura && start_verse > end_verse) {
            return Err(CalculatingLinesError::WrongBoundary);
        }

        let start_verse_data = self.navigator
            .find_verse(start_sura, start_verse)
            .map_err(|_| CalculatingLinesError::WrongBoundary)?.0;
        let end_verse_data = self.navigator
            .find_verse(end_sura, end_verse)
            .map_err(|_| CalculatingLinesError::WrongBoundary)?.0;

        let mut navigator = VersesNavigator::new(self.mushaf.clone(), self.quran_metadata.clone());
        navigator.reset_position(start_verse_data.sura, start_verse_data.number);

        let mut lines = 0.0;
        loop {
            lines = ((lines + navigator.current_verse().lines) * 100.0).round() / 100.0;
            if *navigator.current_verse() == *end_verse_data {
                break;
            }
            navigator.next_verse(direction);
        }

        Ok(lines)
    }

    fn next_verse(
        &self,
        sura_number: u8,
        verse_number: u16,
        direction: Direction
    ) -> Option<Verse> {
        let mut navigator = VersesNavigator::new(self.mushaf.clone(), self.quran_metadata.clone());
        navigator.reset_position(sura_number, verse_number);
        navigator.next_verse(direction).copied()
    }
}
#[cfg(test)]
mod tests {
    use std::{ path::PathBuf, rc::Rc };
    use colored::Colorize;
    use crate::{
        king_fahad_mushaf::{ JsonVerse, KingFahadMushaf },
        mushaf_engine::{ base_mushaf_engine::BaseMushafEngine, IMushafEngine },
        navigation::{ Direction, NavigationResult, VersesNavigator },
    };

    fn setup_engine() -> BaseMushafEngine {
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
        BaseMushafEngine::new(mushaf)
    }

    #[test]
    fn navigate_15_lines() {
        let engine = setup_engine();
        let lines = 15.1f32;
        let result = engine
            .navigate(lines, 5, 3, Direction::Upwards)
            .expect("expect `navigate` to succeed");
        println!("{:16}{}", "Lines: ".bold().cyan(), lines.to_string().yellow().bold());
        println!("{result}");
        let NavigationResult {
            distance_moved,
            end_of_page,
            end_of_sura,
            overflow,
            remaining_distance: _,
            verse,
        } = result;

        assert!((distance_moved - 15.0).abs() < f32::EPSILON);
        assert_eq!(verse.sura, 5);
        assert_eq!(verse.number, 5);
        assert!(end_of_page.is_some());
        assert!(end_of_sura.is_some());
        assert!(overflow.is_some());
        let overflow = overflow.expect("expect `overflow` not to be None");
        let end_of_page = end_of_page.expect("expect `end_of_page` not to be None");
        let end_of_sura = end_of_sura.expect("expect `end_of_sura` not to be None");
        assert!((overflow.overflow_lines - 7.9f32).abs() < f32::EPSILON);
        assert!((end_of_page.lines_distance - -0.1).abs() < f32::EPSILON);
        assert!((end_of_sura.lines_distance - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn navigation_overflow() {
        let engine = setup_engine();

        let v = engine
            .navigate(14.7f32, 114, 1, Direction::Upwards)
            .expect("expect `navigate` to succeed");

        println!("{v}");
        assert!(v.overflow.is_some());
        assert!(v.end_of_sura.is_some());
        assert_eq!(
            v.overflow.expect("expect `overflow` not to be None").overflowed_verse,
            v.end_of_sura.expect("expect `end_of_sura` not to be None").last_verse
        );
        assert_eq!(v.verse.sura, 112);
        assert_eq!(v.verse.number, 3);
    }

    #[test]
    fn navigate_zero_lines() {
        let engine = setup_engine();

        // First verse of Al-Fatiha
        let result = engine
            .navigate(0.0, 1, 1, Direction::Downwards)
            .expect("expect `navigate` to succeed");
        assert_eq!(result.verse.sura, 1);
        assert_eq!(result.verse.number, 1);
    }

    #[test]
    fn calculating_lines() {
        let engine = setup_engine();
        // let navigator = VersesNavigator::new(engine.mushaf.clone(), engine.quran_metadata.clone());
        let lines = engine.calculate_lines(1, 1, 114, 6, Direction::Downwards);
        assert!(lines.is_ok());
        // assert_eq!(lines, engine.)
    }
}
