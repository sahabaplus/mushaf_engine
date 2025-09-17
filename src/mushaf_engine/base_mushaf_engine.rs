use crate::{ mushaf::{ Mushaf, QuranMetadata, SuraInfo, Verse }, navigation::* };
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
        let navigator = VersesNavigator::builder(mushaf.clone(), quran_metadata.clone());
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

    fn is_wrong_direction(
        start: impl Into<VersePosition>,
        end: impl Into<VersePosition>,
        direction: Direction
    ) -> bool {
        let (start, end) = (start.into(), end.into());
        let internal_wrong_direction = start.sura() == end.sura() && start.verse() > end.verse();
        let external_wrong_direction = match direction {
            Direction::Downwards => start.sura() > end.sura(),
            Direction::Upwards => start.sura() < end.sura(),
        };

        internal_wrong_direction || external_wrong_direction
    }

    fn prefer_last_of_sura(
        &self,
        last_of_sura: Option<LastVerseResult>,
        current_verse: &Verse,
        direction: Direction,
        settings: NavigationSettings
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
            .find_verse(VersePosition::new(sura.number, sura.total_verses))
            .expect("Last verse of sura should exist").0;

        let lines_distance =
            self
                .calculate_lines(*current_verse, *last_verse, direction, settings)
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
        direction: Direction,
        settings: NavigationSettings
    ) -> Option<LastVerseResult> {
        if let Some(existing_last) = &last_of_page {
            if *current_verse == existing_last.last_verse {
                return last_of_page;
            }
        }

        let (_, page, _idx) = self.navigator.find_verse(*current_verse).ok()?;
        let new_last_of_page = self.mushaf.get_page(page)?.verses().last()?;

        if Self::is_wrong_direction(*current_verse, *new_last_of_page, direction) {
            return last_of_page;
        }

        let lines_distance = self
            .calculate_lines(*current_verse, *new_last_of_page, direction, settings)
            .ok()?;

        if let Some(existing_last) = last_of_page {
            if existing_last.lines_distance.abs() + 1.0 < lines_distance.abs() {
                return Some(existing_last);
            }
        }

        Some(LastVerseResult::new(lines_distance, *new_last_of_page))
    }

    fn create_navigator(
        &self,
        settings: NavigationSettings,
        direction: Direction
    ) -> VersesNavigator {
        VersesNavigator::new(self.mushaf.clone(), self.quran_metadata.clone(), settings, direction)
    }
}

impl IMushafEngine for BaseMushafEngine {
    fn navigate(
        &self,
        lines: f32,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings
    ) -> Result<NavigationResult, NavigationError> {
        if lines < 0.0 {
            return Err(NavigationError::NegativeLines);
        }

        // If no lines to navigate, find and return the current verse
        if lines == 0.0 {
            if let Ok((verse, ..)) = self.navigator.find_verse(from) {
                return Ok(NavigationResult::new_normal(*verse, 0.0));
            }
            return Err(NavigationError::InvalidVerse);
        }

        // Find starting position
        let Ok((verse, ..)) = self.navigator.find_verse(from) else {
            return Err(NavigationError::InvalidVerse);
        };

        let mut remaining_lines = lines;

        let mut navigator = self.create_navigator(settings, direction);
        navigator.reset_position(*verse);

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
                navigator.next_verse();
            } else {
                break;
            }
        }

        let last_of_sura = self.prefer_last_of_sura(
            last_of_sura,
            &previous_verse,
            direction,
            settings
        );
        let last_of_page = self.prefer_last_of_page(
            last_of_page,
            &previous_verse,
            direction,
            settings
        );

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
        start: impl Into<VersePosition>,
        end: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings
    ) -> Result<f32, CalculatingLinesError> {
        let (start, end) = (start.into(), end.into());
        let is_wrong_direction = Self::is_wrong_direction(start, end, direction);
        if is_wrong_direction {
            return Err(CalculatingLinesError::WrongBoundary);
        }

        let (start_verse_data, ..) = self.navigator
            .find_verse(start)
            .map_err(|_| CalculatingLinesError::WrongBoundary)?;
        let (end_verse_data, ..) = self.navigator
            .find_verse(end)
            .map_err(|_| CalculatingLinesError::WrongBoundary)?;

        let mut navigator = self.create_navigator(settings, direction);
        navigator.reset_position(start);

        let mut lines = 0.0;
        loop {
            let verse_lines = navigator.calculate_verse_lines(navigator.current_verse());
            lines = ((lines + verse_lines) * 100.0).round() / 100.0;
            if navigator.current_verse() == end_verse_data {
                break;
            }
            navigator.next_verse();
        }

        Ok(lines)
    }

    fn next_verse(
        &self,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings
    ) -> Option<Verse> {
        let mut navigator = self.create_navigator(settings, direction);
        navigator.reset_position(from);
        navigator.next_verse().copied()
    }
}
#[cfg(test)]
mod tests {
    use std::{ path::PathBuf, rc::Rc };
    use colored::Colorize;
    use crate::{
        king_fahad_mushaf::{ JsonVerse, KingFahadMushaf },
        mushaf_engine::{ base_mushaf_engine::BaseMushafEngine, IMushafEngine },
        navigation::{ Direction, NavigationResult, VersePosition, VersesNavigator },
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
            .navigate(lines, VersePosition::new(5, 3), Direction::Upwards, Default::default())
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
            .navigate(14.7f32, VersePosition::new(114, 1), Direction::Upwards, Default::default())
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
            .navigate(0.0, VersePosition::start(), Direction::Downwards, Default::default())
            .expect("expect `navigate` to succeed");
        assert_eq!(result.verse.sura, 1);
        assert_eq!(result.verse.number, 1);
    }

    #[test]
    fn calculating_lines() {
        let engine = setup_engine();
        let lines = engine.calculate_lines(
            VersePosition::start(),
            VersePosition::end(),
            Direction::Downwards,
            Default::default()
        );
        assert!(lines.is_ok());
    }
}
