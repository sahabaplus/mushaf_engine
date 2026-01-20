use crate::{
    mushaf::{Mushaf, QuranMetadata, SuraInfo, Verse},
    navigation::*,
};
use std::rc::Rc;

use super::IMushafEngine;

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
        Self {
            mushaf,
            quran_metadata,
            navigator,
        }
    }

    /// Get metadata about a specific Sura
    ///
    /// # Errors
    ///
    /// Returns `LookupError` if the sura number is invalid or not found
    pub fn get_sura_info(&self, sura_number: u8) -> Result<&SuraInfo, LookupError> {
        self.quran_metadata.get_sura_info(sura_number)
    }

    fn prefer_last_of_sura(
        &self,
        last_of_sura: Option<LastVerseResult>,
        current_verse: &Verse,
        direction: Direction,
    ) -> LastVerseResult {
        if let Some(last_result) = &last_of_sura
            && current_verse.sura == last_result.last_verse.sura
        {
            return last_result.clone();
        }

        let sura = self
            .quran_metadata
            .get_sura_info(current_verse.sura)
            .expect("Sura should exist for current verse");
        let last_verse = self
            .navigator
            .find_verse(VersePosition::new(sura.number, sura.total_verses))
            .expect("Last verse of sura should exist")
            .0;

        let lines_distance = self
            .calculate_lines(*current_verse, *last_verse, direction, Default::default())
            .expect("Lines calculation should succeed")
            - current_verse.lines;

        if let Some(existing_last) = last_of_sura
            && existing_last.lines_distance.abs() + 3.0 < lines_distance.abs()
        {
            return existing_last;
        }

        LastVerseResult::new(lines_distance, *last_verse)
    }

    fn prefer_last_of_page(
        &self,
        last_of_page: Option<LastVerseResult>,
        current_verse: &Verse,
        direction: Direction,
    ) -> Option<LastVerseResult> {
        if let Some(existing_last) = &last_of_page
            && *current_verse == existing_last.last_verse
        {
            return last_of_page;
        }

        let (_, page, _idx) = self.navigator.find_verse(*current_verse).ok()?;
        let new_last_of_page = self.mushaf.get_page(page)?.verses().last()?;

        if VersesNavigator::is_wrong_direction(*current_verse, *new_last_of_page, direction) {
            return last_of_page;
        }

        let lines_distance = self
            .calculate_lines(
                *current_verse,
                *new_last_of_page,
                direction,
                Default::default(),
            )
            .ok()?;

        if let Some(existing_last) = last_of_page
            && existing_last.lines_distance.abs() + 1.0 < lines_distance.abs()
        {
            return Some(existing_last);
        }

        Some(LastVerseResult::new(lines_distance, *new_last_of_page))
    }

    fn create_navigator(
        &self,
        settings: NavigationSettings,
        direction: Direction,
    ) -> VersesNavigator {
        VersesNavigator::new(
            self.mushaf.clone(),
            self.quran_metadata.clone(),
            settings,
            direction,
        )
    }
}

impl IMushafEngine for BaseMushafEngine {
    fn navigate(
        &self,
        lines: f32,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
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
        navigator.reset_position(*verse).map_err(|_| NavigationError::OutOfBounds)?;

        let mut overflow: Option<OverflowResult> = None;
        let mut previous_verse = *navigator.current_verse();

        let mut last_of_page: Option<LastVerseResult> = None;
        let mut last_of_sura: Option<LastVerseResult> = None;

        loop {
            let current_verse = navigator.current_verse();
            // We looped twice and stuck at the same verse
            if &previous_verse == current_verse && (lines - remaining_lines).abs() > f32::EPSILON {
                break;
            }
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

            if remaining_lines > f32::EPSILON {
                navigator.next_verse();
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
        start: impl Into<VersePosition>,
        end: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> Result<f32, CalculatingLinesError> {
        let (start, end) = (start.into(), end.into());
        let is_wrong_direction = VersesNavigator::is_wrong_direction(start, end, direction);
        let out_of_bounds = self.navigator.is_out_of_bounds(start);
        if is_wrong_direction && out_of_bounds {
            return Err(CalculatingLinesError::WrongBoundary);
        }

        let (start_verse_data, ..) = self
            .navigator
            .find_verse(start)
            .map_err(|_| CalculatingLinesError::WrongBoundary)?;
        let (end_verse_data, ..) = self
            .navigator
            .find_verse(end)
            .map_err(|_| CalculatingLinesError::WrongBoundary)?;

        let mut navigator = self.create_navigator(settings, direction);
        navigator.reset_position(start);

        let mut lines = 0.0;
        loop {
            let verse_lines = navigator.calculate_verse_lines(navigator.current_verse());
            lines += verse_lines;
            if navigator.current_verse() == end_verse_data {
                break;
            }

            // Check if next_verse() returns None to prevent infinite loop
            if navigator.next_verse().is_none() {
                // If we couldn't reach the end verse, return an error
                return Err(CalculatingLinesError::WrongBoundary);
            }
        }

        // Round lines to 2 decimal places
        lines = (lines * 100.0).round() / 100.0;
        Ok(lines)
    }

    fn next_verse(
        &self,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> Option<Verse> {
        let mut navigator = self.create_navigator(settings, direction);
        navigator.reset_position(from);
        navigator.next_verse().copied()
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        king_fahad_mushaf::{JsonVerse, KingFahadMushaf},
        mushaf_engine::{IMushafEngine, base_mushaf_engine::BaseMushafEngine},
        navigation::{
            Direction, LookupError, NavigationError, NavigationResult, NavigationSettings,
            VersePosition, VersesNavigator,
        },
    };
    use colored::Colorize;
    use std::{path::PathBuf, rc::Rc};

    fn setup_engine() -> BaseMushafEngine {
        let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data_path.push("data");
        data_path.push("king_fahad_mushaf.json");

        let mushaf = Rc::new({
            let path = data_path.to_str().expect("expect `data_path` to be a valid string");
            // Load from provided JSON path
            let file_content =
                std::fs::read_to_string(path).expect("Failed to read mushaf data file");

            let pages: Vec<Vec<JsonVerse>> =
                serde_json::from_str(&file_content).expect("Failed to parse mushaf JSON data");

            KingFahadMushaf::create_mushaf_from_pages(pages)
        });
        BaseMushafEngine::new(mushaf)
    }

    #[test]
    fn navigate_15_lines() {
        let engine = setup_engine();
        let lines = 15.1f32;
        let result = engine
            .navigate(
                lines,
                VersePosition::new(5, 3),
                Direction::Upwards,
                Default::default(),
            )
            .expect("expect `navigate` to succeed");
        println!(
            "{:16}{}",
            "Lines: ".bold().cyan(),
            lines.to_string().yellow().bold()
        );
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
            .navigate(
                14.7f32,
                VersePosition::new(114, 1),
                Direction::Upwards,
                NavigationSettings::builder(),
            )
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
            .navigate(
                0.0,
                VersePosition::start(),
                Direction::Downwards,
                Default::default(),
            )
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
            Default::default(),
        );
        assert!(lines.is_ok());
    }

    #[test]
    fn multiple_iterations() {
        let engine = setup_engine();
        let settings = NavigationSettings::builder().ignore_sura_header(true);
        let lines = engine.calculate_lines(
            VersePosition::start(),
            VersePosition::end(),
            Direction::Downwards,
            settings,
        );
        assert!(lines.is_ok());
        println!("{lines:?}");
        let whole_mushaf_lines = lines.expect("expect `lines` not to be None");

        let navigate_by_whole_mushaf_ratio = |ratio: f32, additional_lines: f32| {
            engine
                .navigate(
                    whole_mushaf_lines * ratio + additional_lines,
                    VersePosition::start(),
                    Direction::Downwards,
                    settings.iteration_limit(1),
                )
                .expect("expect `navigate` to succeed")
        };

        // One full cycle
        let result = navigate_by_whole_mushaf_ratio(1.0, 0.0);
        assert_eq!(VersePosition::end(), result.verse);

        // One full cycle plus small overflow
        let result = navigate_by_whole_mushaf_ratio(1.0, 0.1);
        assert_eq!(VersePosition::end(), result.verse);
        assert!(result.overflow.is_some());
        let overflow = result.overflow.expect("expect `overflow` not to be None");
        // Should round back to start
        assert_eq!(VersePosition::start(), overflow.overflowed_verse);

        // One and a half cycle
        let result = navigate_by_whole_mushaf_ratio(1.5, 0.0);
        assert_eq!(result.verse.sura, 18); // Surat Al-Kahf
        assert!((30..=39).contains(&result.verse.number));

        // Two full cycles
        let result = navigate_by_whole_mushaf_ratio(2.0, 0.0);
        assert_eq!(VersePosition::end(), result.verse);

        // Two full cycles plus small overflow
        let result = navigate_by_whole_mushaf_ratio(2.0, 0.1);
        assert_eq!(VersePosition::end(), result.verse);
        assert!(result.end_of_page.is_some());
        assert!(result.end_of_sura.is_some());
        let end_of_page = result.end_of_page.expect("expect `end_of_page` not to be None");
        let end_of_sura = result.end_of_sura.expect("expect `end_of_sura` not to be None");
        // Stick at the end
        assert_eq!(VersePosition::end(), end_of_page.last_verse);
        assert_eq!(VersePosition::end(), end_of_sura.last_verse);
    }

    #[test]
    fn bounded_navigation() {
        let engine = setup_engine();
        let start_bound = VersePosition::new(2, 1);
        let end_bound = VersePosition::new(2, 50);
        let settings =
            NavigationSettings::builder().upper_bound(start_bound).lower_bound(end_bound);
        let result = engine.navigate(10.0, VersePosition::start(), Default::default(), settings);
        assert!(result.is_err());
        let err = result.expect_err("expect `err` not to be None");
        assert!(err == NavigationError::OutOfBounds);

        let result = engine.navigate(0.0, start_bound, Default::default(), settings);
        assert!(result.is_ok());
        let verse = result.expect("expect `result` not to be None").verse;
        assert_eq!(start_bound, verse);

        // Massive overflow
        let result = engine.navigate(1000.0, start_bound, Default::default(), settings);
        assert!(result.is_ok());
        let verse = result.expect("expect `result` not to be None").verse;
        assert_eq!(end_bound, verse);

        // Multiple iterations
        let bounds_lines =
            engine.calculate_lines(start_bound, end_bound, Default::default(), settings);
        assert!(bounds_lines.is_ok());
        let bounds_lines = bounds_lines.expect("expect `bounds_lines` not to be None");
        let result = engine.navigate(
            bounds_lines * 2.0 + 15.0,
            start_bound,
            Default::default(),
            settings.iteration_limit(2),
        );
        assert!(result.is_ok());
        let verse = result.expect("expect `result` not to be None").verse;
        assert_eq!(verse.sura, 2);
        assert_eq!(verse.number, 10);
    }

    #[test]
    fn calculate_lines_with_bounds() {
        let engine = setup_engine();
        let start_bound = VersePosition::new(2, 1);
        let end_bound = VersePosition::new(2, 286);
        let settings = NavigationSettings::builder()
            .iteration_limit(2)
            .upper_bound(start_bound)
            .lower_bound(end_bound);

        let full_sura_lines_metadata = engine
            .quran_metadata
            .get_sura_info(2)
            .expect("expect `full_sura` not to be None")
            .lines_with_header;
        let full_sura_in_normal_direction = engine.calculate_lines(
            VersePosition::new(2, 1),
            VersePosition::new(2, 286),
            Direction::Downwards,
            Default::default(),
        );
        let lines = engine.calculate_lines(
            VersePosition::new(2, 280),
            VersePosition::new(2, 279),
            Direction::Downwards,
            settings,
        );
        assert!(full_sura_in_normal_direction.is_ok());
        assert!(lines.is_ok());

        let full_sura_in_normal_direction = full_sura_in_normal_direction
            .expect("expect `full_sura_in_normal_direction` not to be None");
        let lines = lines.expect("expect `lines` not to be None");

        // They should be the same
        assert_eq!(full_sura_in_normal_direction, lines);
        assert_eq!(
            full_sura_in_normal_direction,
            (full_sura_lines_metadata * 10.0).round() / 10.0
        );
    }

    #[test]
    fn calculate_lines_with_excluding_bounds() {
        let engine = setup_engine();

        // Test 1: Exclude verses 10-20 in sura 2 (exclusive bounds) - Downwards direction
        // Excluding bounds: lower_bound=(2,10), upper_bound=(2,20)
        // This means: exclude verses strictly between (2,10) and (2,20), i.e., verses 11-19
        let excluding_settings = NavigationSettings::builder()
            .upper_bound(VersePosition::new(2, 20)) // upper > lower = excluding mode
            .lower_bound(VersePosition::new(2, 10));

        // Calculate lines from (2,1) to (2,30) with excluding bounds
        // Expected: include verses 1-10, skip 11-19, include 20-30
        let lines_with_excluding = engine
            .calculate_lines(
                VersePosition::new(2, 1),
                VersePosition::new(2, 30),
                Direction::Downwards,
                excluding_settings,
            )
            .expect("Should calculate lines");

        // Directly calculate what should be included: verses 1-10 and 20-30
        let lines_1_to_10 = engine
            .calculate_lines(
                VersePosition::new(2, 1),
                VersePosition::new(2, 10),
                Direction::Downwards,
                Default::default(),
            )
            .expect("Should calculate");

        let lines_20_to_30 = engine
            .calculate_lines(
                VersePosition::new(2, 20),
                VersePosition::new(2, 30),
                Direction::Downwards,
                Default::default(),
            )
            .expect("Should calculate");

        let expected_lines = lines_1_to_10 + lines_20_to_30;
        assert!(
            (lines_with_excluding - expected_lines).abs() < 0.1,
            "Downwards: Expected {} lines (verses 1-10 + 20-30), got {} lines",
            expected_lines,
            lines_with_excluding
        );

        // Test 2: Downwards with different range
        // Test excluding verses 50-100 in sura 2
        let excluding_settings_2 = NavigationSettings::builder()
            .upper_bound(VersePosition::new(2, 100))
            .lower_bound(VersePosition::new(2, 50));

        let lines_with_excluding_2 = engine
            .calculate_lines(
                VersePosition::new(2, 1),
                VersePosition::new(2, 150),
                Direction::Downwards,
                excluding_settings_2,
            )
            .expect("Should calculate lines");

        // Calculate what should be included: verses 1-50 and 100-150
        let lines_1_to_50 = engine
            .calculate_lines(
                VersePosition::new(2, 1),
                VersePosition::new(2, 50),
                Direction::Downwards,
                Default::default(),
            )
            .expect("Should calculate");

        let lines_100_to_150 = engine
            .calculate_lines(
                VersePosition::new(2, 100),
                VersePosition::new(2, 150),
                Direction::Downwards,
                Default::default(),
            )
            .expect("Should calculate");

        let expected_lines_2 = lines_1_to_50 + lines_100_to_150;
        assert!(
            (lines_with_excluding_2 - expected_lines_2).abs() < 0.1,
            "Downwards 2: Expected {} lines (verses 1-50 + 100-150), got {} lines",
            expected_lines_2,
            lines_with_excluding_2
        );

        // Test 3: Cross-sura excluding bounds - Downwards direction
        // Exclude verses between (1,5) and (2,5) - should skip verses 1:6-7 and 2:1-4
        let cross_sura_excluding = NavigationSettings::builder()
            .upper_bound(VersePosition::new(2, 5))
            .lower_bound(VersePosition::new(1, 5));

        // Calculate from (1,1) to (2,10) with excluding bounds
        // Expected: include (1,1-5), skip (1,6-7) and (2,1-4), include (2,5-10)
        let lines_cross_sura_down = engine
            .calculate_lines(
                VersePosition::new(1, 1),
                VersePosition::new(2, 10),
                Direction::Downwards,
                cross_sura_excluding,
            )
            .expect("Should calculate lines");

        // Directly calculate what should be included
        let sura1_included = engine
            .calculate_lines(
                VersePosition::new(1, 1),
                VersePosition::new(1, 5),
                Direction::Downwards,
                Default::default(),
            )
            .expect("Should calculate");

        let sura2_included = engine
            .calculate_lines(
                VersePosition::new(2, 5),
                VersePosition::new(2, 10),
                Direction::Downwards,
                Default::default(),
            )
            .expect("Should calculate");

        let expected_cross_sura_down = sura1_included + sura2_included;
        assert!(
            (lines_cross_sura_down - expected_cross_sura_down).abs() < 0.1,
            "Cross-sura Downwards: Expected {} lines ((1,1-5) + (2,5-10)), got {} lines",
            expected_cross_sura_down,
            lines_cross_sura_down
        );

        // Test 4: Another cross-sura excluding test
        // Exclude verses between (2,100) and (3,50)
        let cross_sura_excluding_2 = NavigationSettings::builder()
            .upper_bound(VersePosition::new(3, 50))
            .lower_bound(VersePosition::new(2, 100));

        let lines_cross_sura_2 = engine
            .calculate_lines(
                VersePosition::new(2, 50),
                VersePosition::new(3, 100),
                Direction::Downwards,
                cross_sura_excluding_2,
            )
            .expect("Should calculate lines");

        // Calculate what should be included: (2,50-100) + (3,50-100)
        let sura2_part = engine
            .calculate_lines(
                VersePosition::new(2, 50),
                VersePosition::new(2, 100),
                Direction::Downwards,
                Default::default(),
            )
            .expect("Should calculate");

        let sura3_part = engine
            .calculate_lines(
                VersePosition::new(3, 50),
                VersePosition::new(3, 100),
                Direction::Downwards,
                Default::default(),
            )
            .expect("Should calculate");

        let expected_cross_sura_2 = sura2_part + sura3_part;
        assert!(
            (lines_cross_sura_2 - expected_cross_sura_2).abs() < 0.1,
            "Cross-sura 2: Expected {} lines ((2,50-100) + (3,50-100)), got {} lines",
            expected_cross_sura_2,
            lines_cross_sura_2
        );

        // Test 5: Verify excluded verses are actually skipped
        // For cross-sura excluding, verify that excluded range has different line count
        let excluded_range_down = engine
            .calculate_lines(
                VersePosition::new(1, 6),
                VersePosition::new(2, 4),
                Direction::Downwards,
                Default::default(),
            )
            .expect("Should calculate");

        // The excluded range should NOT be included in the result
        let full_range_down = engine
            .calculate_lines(
                VersePosition::new(1, 1),
                VersePosition::new(2, 10),
                Direction::Downwards,
                Default::default(),
            )
            .expect("Should calculate");

        // With excluding, we should have: full_range - excluded_range
        let expected_with_excluding = full_range_down - excluded_range_down;
        assert!(
            (lines_cross_sura_down - expected_with_excluding).abs() < 0.1,
            "Verification: Expected {} lines (full - excluded), got {} lines",
            expected_with_excluding,
            lines_cross_sura_down
        );
    }
}
