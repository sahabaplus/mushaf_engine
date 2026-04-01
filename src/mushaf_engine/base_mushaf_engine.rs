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
            .calculate_lines(current_verse, last_verse, direction, Default::default())
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

        let (_, page, _idx) = self.navigator.find_verse(current_verse).ok()?;
        let new_last_of_page = self.mushaf.get_page(page)?.verses().last()?;

        if VersesNavigator::is_wrong_direction(current_verse, new_last_of_page, direction) {
            return last_of_page;
        }

        let lines_distance = self
            .calculate_lines(
                current_verse,
                new_last_of_page,
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
        navigator.reset_position(verse).map_err(|_| NavigationError::OutOfBounds)?;

        let mut overflow: Option<OverflowResult> = None;
        let mut previous_verse = *navigator.current_verse();

        let mut last_of_page: Option<LastVerseResult> = None;
        let mut last_of_sura: Option<LastVerseResult> = None;

        // Track cycles (passes through initial verse) and boundary crossings
        let initial_verse = *navigator.current_verse();
        let mut cycles_completed = 0u32;
        let mut crossed_boundaries = false;
        let mut has_moved = false;

        // Track cycle distance (lines accumulated in the current cycle)
        let mut lines_in_current_cycle = 0.0;
        let mut cycle_distance = 0.0; // Set when first cycle completes

        loop {
            let current_verse = navigator.current_verse();
            // We looped twice and stuck at the same verse
            if &previous_verse == current_verse && (lines - remaining_lines).abs() > f32::EPSILON {
                break;
            }

            // Track cycles: count passes through the initial verse (after at least one move)
            // Note: When a cycle is detected, we continue the loop and count this verse's lines
            // This ensures consistency with calculate_lines cycle distance calculation
            if has_moved && current_verse == &initial_verse {
                cycles_completed += 1;
                if cycle_distance == 0.0 {
                    // Capture cycle distance on first cycle completion
                    cycle_distance = lines_in_current_cycle;
                }
                lines_in_current_cycle = 0.0; // Reset for next cycle
            }

            let current_sura_info = self
                .quran_metadata
                .get_sura_info(current_verse.sura)
                .expect("Current verse sura should exist");
            let verse_lines = navigator.calculate_verse_lines(current_verse);

            // Accumulate lines for cycle distance tracking
            lines_in_current_cycle += verse_lines;

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
                // Check return value - break if boundary reached
                if navigator.next_verse().is_none() {
                    break;
                }
                if navigator.crossed_boundaries {
                    crossed_boundaries = true;
                }
                has_moved = true;
            } else {
                break;
            }
        }

        let last_of_sura = self.prefer_last_of_sura(last_of_sura, &previous_verse, direction);
        let last_of_page = self.prefer_last_of_page(last_of_page, &previous_verse, direction);

        Ok(NavigationResult::new(
            previous_verse,
            overflow,
            last_of_page,
            Some(last_of_sura),
            lines - remaining_lines,
            CycleInfo::new(cycles_completed, crossed_boundaries, cycle_distance),
        ))
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

        let _ = self
            .navigator
            .find_verse(start)
            .map_err(|_| CalculatingLinesError::WrongBoundary)?;
        let (end_verse_data, ..) = self
            .navigator
            .find_verse(end)
            .map_err(|_| CalculatingLinesError::WrongBoundary)?;

        // Calculate direct distance from start to end
        let mut navigator = self.create_navigator(settings, direction);
        let _ = navigator.reset_position(start);

        let mut direct_lines = 0.0;
        loop {
            let verse_lines = navigator.calculate_verse_lines(navigator.current_verse());
            direct_lines += verse_lines;
            if navigator.current_verse() == end_verse_data {
                break;
            }

            // Check if next_verse() returns None to prevent infinite loop
            if navigator.next_verse().is_none() {
                // If we couldn't reach the end verse, return an error
                return Err(CalculatingLinesError::WrongBoundary);
            }
        }

        let lines = (direct_lines * 100.0).round() / 100.0;
        Ok(lines)
    }

    fn next_verse(
        &self,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> Option<Verse> {
        let mut navigator = self.create_navigator(settings, direction);
        let _ = navigator.reset_position(from);
        navigator.next_verse().copied()
    }

    fn previous_verse(
        &self,
        from: impl Into<VersePosition>,
        direction: Direction,
        settings: NavigationSettings,
    ) -> Option<Verse> {
        let mut navigator = self.create_navigator(settings, direction);
        let _ = navigator.reset_position(from);
        navigator.previous_verse().copied()
    }

    fn is_out_of_bounds(
        &self,
        verse: impl Into<VersePosition>,
        settings: NavigationSettings,
    ) -> bool {
        let navigator = self.create_navigator(settings, Direction::Downwards);
        navigator.is_out_of_bounds(verse.into())
    }

    fn get_start_bound(
        &self,
        direction: Direction,
        settings: NavigationSettings,
    ) -> VersePosition {
        let navigator = self.create_navigator(settings, direction);
        navigator.get_start_bound()
    }

    fn get_end_bound(
        &self,
        direction: Direction,
        settings: NavigationSettings,
    ) -> VersePosition {
        let navigator = self.create_navigator(settings, direction);
        navigator.get_end_bound()
    }

    fn find_verse(&self, position: impl Into<VersePosition>) -> Option<Verse> {
        self.navigator
            .find_verse(position)
            .ok()
            .map(|(verse, ..)| *verse)
    }
}

#[cfg(test)]
#[path = "base_mushaf_engine_tests.rs"]
mod tests;
