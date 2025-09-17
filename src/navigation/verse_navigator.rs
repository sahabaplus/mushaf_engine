use crate::{
    mushaf::{ Mushaf, QuranMetadata, Verse },
    navigation::{ LookupError, NavigationSettings, VersePosition },
};
use std::rc::Rc;

use super::Direction;

/// A navigator for moving through verses in the Mushaf with bounded iteration support.
///
/// `VersesNavigator` provides controlled navigation through Quranic verses with support
/// for bounded ranges and iteration limits. It automatically handles direction changes
/// and maintains iteration state for cycling through defined ranges.
///
/// # Key Features
///
/// ## Bounded Navigation
/// - Navigate within specific verse ranges using `NavigationBounds`
/// - Control iteration limits to cycle through ranges multiple times
/// - Automatic bounds reversal when changing direction
///
/// ## Iteration Behavior
/// The navigator tracks iteration count and manages cycling behavior:
/// - When reaching `end_position` with remaining iterations: resets to `start_position`
/// - When reaching `end_position` with no remaining iterations: returns `None`
/// - Iteration count is incremented each time a cycle completes
///
/// ## Direction Support
/// - **Downwards**: Navigate from lower sura numbers to higher (1→114)
/// - **Upwards**: Navigate from higher sura numbers to lower (114→1)
/// - Bounds are automatically reversed when direction changes
///
/// # Iteration Limits and Bounds
///
/// ## How Iteration Works
/// 1. Navigator starts at `start_position`
/// 2. Moves through verses until reaching `end_position`
/// 3. If `remaining_iterations > 0`: increment count and reset to `start_position`
/// 4. If `remaining_iterations = 0`: return `None` (stop navigation)
///
/// ## Practical Examples
///
/// ### Reading Al-Fatiha Three Times
/// ```ignore
/// let navigator = VersesNavigator::builder(mushaf, metadata)
///     .start_position(VersePosition::new(1, 1))
///     .end_position(VersePosition::new(1, 7))
///     .iteration_limit(3)  // Read Al-Fatiha 3 times
///     .direction(Direction::Downwards);
/// ```
///
/// ### Page Layout Calculation
/// ```ignore
/// let navigator = VersesNavigator::builder(mushaf, metadata)
///     .start_position(VersePosition::new(2, 1))
///     .end_position(VersePosition::new(2, 50))
///     .iteration_limit(0)  // No cycling for layout
///     .ignore_sura_header(true);  // Exclude headers for precise calculation
/// ```
///
/// ### Memorization Practice
/// ```ignore
/// let navigator = VersesNavigator::builder(mushaf, metadata)
///     .start_position(VersePosition::new(67, 1))
///     .end_position(VersePosition::new(67, 30))
///     .iteration_limit(5)  // Practice 5 times
///     .direction(Direction::Upwards);  // Read backwards
/// ```
///
/// # When to Use Different Settings
///
/// ## Use iteration limits when:
/// - **Repeating recitation**: Reading the same sura multiple times
/// - **Memorization practice**: Cycling through specific verses repeatedly
/// - **Controlled navigation**: Limiting how far navigation can go
/// - **Page layout calculations**: Measuring content within bounds
///
/// ## Don't use iteration limits when:
/// - **Linear reading**: Moving through the Quran sequentially
/// - **Search operations**: Looking for specific content
/// - **One-time navigation**: Moving from point A to point B once
///
/// ## Use `ignore_sura_header: true` when:
/// - **Accuracy**: Calculating exact verse lines without headers
///
/// ## Use `ignore_sura_header: false` when:
/// - **Page layout calculations**: Page lines end up having 15 lines wither containing headers of suras or normal full page of verses
pub struct VersesNavigator {
    mushaf: Rc<Mushaf>,
    quran_metadata: Rc<QuranMetadata>,
    current_page_idx: usize,
    current_verse_idx: usize,
    settings: NavigationSettings,
    direction: Direction,
    /// Current iteration count within the bounded range.
    ///
    /// This tracks how many complete cycles through the bounded range
    /// have been completed. It's incremented each time the navigator
    /// reaches the end_position and resets to start_position.
    pub iteration_count: u32,
}

impl VersesNavigator {
    /// Creates a new `VersesNavigator` with specified settings and direction.
    ///
    /// # Arguments
    /// * `mushaf` - The Mushaf containing verse data
    /// * `metadata` - Quran metadata for sura information
    /// * `settings` - Navigation settings including bounds and iteration limits
    /// * `direction` - Navigation direction (automatically adjusts bounds)
    ///
    /// # Bounds and Direction
    /// The bounds are automatically reversed when using `Direction::Upwards` to ensure
    /// navigation always flows from start to end position regardless of direction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationSettings, Direction, VersePosition};
    ///
    /// let settings = NavigationSettings::builder()
    ///     .start_position(VersePosition::new(1, 1))
    ///     .end_position(VersePosition::new(1, 7))
    ///     .iteration_limit(2);
    ///
    /// ```
    /// ```ignore
    /// let navigator = VersesNavigator::new(mushaf, metadata, settings, Direction::Downwards);
    /// ```
    #[must_use]
    pub fn new(
        mushaf: Rc<Mushaf>,
        metadata: Rc<QuranMetadata>,
        settings: NavigationSettings,
        direction: Direction
    ) -> Self {
        let settings = if
            Self::is_wrong_direction(
                settings.bounds.start_position,
                settings.bounds.end_position,
                direction
            )
        {
            settings.reverse_bounds()
        } else {
            settings
        };

        let mut s = Self {
            mushaf,
            quran_metadata: metadata,
            current_page_idx: 0,
            current_verse_idx: 0,
            direction,
            settings,
            iteration_count: 0,
        };

        s.reset_position(settings.bounds.start_position).expect(
            "expect `reset_position` to succeed"
        );
        s
    }

    /// Creates a builder for `VersesNavigator` with default settings.
    ///
    /// Default settings:
    /// - `direction`: `Direction::Downwards`
    /// - `iteration_limit`: 0 (no cycling)
    /// - `start_position`: (1, 1) - beginning of Al-Fatiha
    /// - `end_position`: (114, 6) - end of An-Nas
    /// - `ignore_sura_header`: false (include headers)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let navigator = VersesNavigator::builder(mushaf, metadata)
    ///     .start_position(VersePosition::new(2, 1))
    ///     .end_position(VersePosition::new(2, 50))
    ///     .iteration_limit(1)
    ///     .direction(Direction::Upwards);
    /// ```
    #[must_use]
    pub fn builder(mushaf: Rc<Mushaf>, metadata: Rc<QuranMetadata>) -> Self {
        Self::new(mushaf, metadata, Default::default(), Default::default())
    }

    pub fn is_wrong_direction(
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

    /// Sets the navigation direction and automatically adjusts bounds.
    ///
    /// When the direction changes, the bounds are automatically reversed to ensure
    /// navigation always flows from start to end position regardless of direction.
    ///
    /// # Arguments
    /// * `direction` - Navigation direction (automatically adjusts bounds)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let navigator = navigator.direction(Direction::Upwards); // Bounds automatically reversed
    /// ```
    #[must_use]
    pub fn direction(mut self, direction: Direction) -> Self {
        // If the direction is reversed, reverse the bounds
        if direction != self.direction {
            self.settings = self.settings.reverse_bounds();
        }
        self.direction = direction;
        self
    }

    /// Sets the complete navigation settings.
    ///
    /// # Arguments
    /// * `settings` - Complete navigation settings including bounds and header handling
    #[must_use]
    pub fn settings(mut self, settings: NavigationSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Sets whether to ignore sura headers in line calculations.
    ///
    /// # Arguments
    /// * `ignore_sura_header` - Whether to exclude sura headers from calculations
    #[must_use]
    pub fn ignore_sura_header(mut self, ignore_sura_header: bool) -> Self {
        self.settings.ignore_sura_header = ignore_sura_header;
        self
    }

    /// Sets the start position for navigation bounds.
    ///
    /// # Arguments
    /// * `start_position` - Starting position for navigation
    #[must_use]
    pub fn start_position(mut self, start_position: impl Into<VersePosition>) -> Self {
        self.settings.bounds.start_position = start_position.into();
        self
    }

    /// Sets the end position for navigation bounds.
    ///
    /// # Arguments
    /// * `end_position` - Ending position for navigation
    #[must_use]
    pub fn end_position(mut self, end_position: impl Into<VersePosition>) -> Self {
        self.settings.bounds.end_position = end_position.into();
        self
    }

    /// Sets the iteration limit for navigation bounds.
    ///
    /// # Arguments
    /// * `iteration_limit` - Maximum number of complete cycles through the range
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let navigator = navigator.iteration_limit(3); // Allow 3 complete cycles
    /// ```
    #[must_use]
    pub fn iteration_limit(mut self, iteration_limit: u32) -> Self {
        self.settings.bounds.iteration_limit = iteration_limit;
        self
    }

    /// Reset the navigator to a specific verse position
    ///
    /// # Errors
    /// * `LookupError::InvalidVerse` if the verse number is invalid
    pub fn reset_position(
        &mut self,
        verse: impl Into<VersePosition>
    ) -> Result<&Verse, LookupError> {
        let verse = verse.into();
        if self.is_out_of_bounds(verse) {
            return Err(LookupError::OutOfBounds);
        }

        let (.., page, idx) = self.find_verse(verse)?;
        self.current_page_idx = (page as usize) - 1;
        self.current_verse_idx = idx as usize;
        Ok(self.current_verse())
    }

    pub fn is_out_of_bounds(&self, verse: VersePosition) -> bool {
        let verse = self.find_verse(verse);
        if verse.is_err() {
            return true;
        }
        let (verse, ..) = verse.expect("expect `verse` not to be None");
        let verse_sura = verse.sura;
        let verse_number = verse.number;
        let start_position = self.settings.bounds.start_position;
        let end_position = self.settings.bounds.end_position;

        let lower_bound_sura = ::std::cmp::min(start_position.sura(), end_position.sura());
        let upper_bound_sura = ::std::cmp::max(start_position.sura(), end_position.sura());
        let is_out_of_bounds_sura: bool =
            verse_sura < lower_bound_sura || verse_sura > upper_bound_sura;
        if is_out_of_bounds_sura {
            return true;
        }

        // Verse sura in the bounds

        if verse_sura == start_position.sura() {
            return verse_number < start_position.verse();
        }
        if verse_sura == end_position.sura() {
            return verse_number > end_position.verse();
        }
        false
    }

    /// Resets the iteration counter to a specific value.
    ///
    /// This method allows you to manually control the iteration count, which is useful
    /// for scenarios where you want to start from a specific iteration state or
    /// reset the counter after modifying bounds.
    ///
    /// # Arguments
    /// * `iterations` - New iteration count value
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Reset to start fresh (no iterations completed)
    /// navigator.reset_iterations(0);
    ///
    /// // Set to indicate 2 iterations have already been completed
    /// navigator.reset_iterations(2);
    /// ```
    pub fn reset_iterations(&mut self, iterations: u32) {
        self.iteration_count = iterations;
    }

    /// Get the current verse
    #[must_use]
    pub fn current_verse(&self) -> &Verse {
        let current_page_verses = self.mushaf.pages[self.current_page_idx].verses();
        &current_page_verses[self.current_verse_idx]
    }

    #[must_use]
    pub fn get_settings(&self) -> &NavigationSettings {
        &self.settings
    }

    /// Moves to the next verse based on direction and iteration bounds.
    ///
    /// This is the core navigation method that handles iteration limits and bounded navigation.
    /// It automatically manages cycling through bounded ranges and respects iteration limits.
    ///
    /// # Iteration Behavior
    ///
    /// The method implements the following iteration logic:
    /// 1. Check if current position equals the `end_position`
    /// 2. If at end position and iterations remain: increment count and reset to `start_position`
    /// 3. If at end position and no iterations remain: return `None` (stop navigation)
    /// 4. Otherwise: move to next verse in the specified direction
    ///
    /// # Return Value
    /// * `Some(&Verse)` - Successfully moved to next verse
    /// * `None` - Reached end of bounds with no remaining iterations
    ///
    /// # Examples
    ///
    /// ## Basic Navigation
    /// ```ignore
    /// let mut navigator = VersesNavigator::builder(mushaf, metadata)
    ///     .start_position(VersePosition::new(1, 1))
    ///     .end_position(VersePosition::new(1, 7))
    ///     .iteration_limit(0); // No cycling
    ///
    /// // Navigate through Al-Fatiha once
    /// while let Some(verse) = navigator.next_verse() {
    ///     println!("{}", verse);
    /// }
    /// ```
    ///
    /// ## Iteration with Cycling
    /// ```ignore
    /// let mut navigator = VersesNavigator::builder(mushaf, metadata)
    ///     .start_position(VersePosition::new(1, 1))
    ///     .end_position(VersePosition::new(1, 7))
    ///     .iteration_limit(2); // Read Al-Fatiha twice
    ///
    /// // Will cycle through Al-Fatiha twice
    /// while let Some(verse) = navigator.next_verse() {
    ///     println!("{}", verse);
    /// }
    /// ```
    ///
    /// ## Direction-Aware Navigation
    /// ```ignore
    /// let mut navigator = VersesNavigator::builder(mushaf, metadata)
    ///     .start_position(VersePosition::new(2, 1))
    ///     .end_position(VersePosition::new(2, 5))
    ///     .direction(Direction::Upwards); // Navigate backwards
    ///
    /// // Will navigate from (2,5) to (2,1) due to automatic bounds reversal
    /// while let Some(verse) = navigator.next_verse() {
    ///     println!("{}", verse);
    /// }
    /// ```
    ///
    /// # Iteration Limits Explained
    ///
    /// - **iteration_limit = 0**: No cycling - stops at end position
    /// - **iteration_limit = 1**: One complete cycle through the range
    /// - **iteration_limit = n**: n complete cycles through the range
    ///
    /// Each time the navigator reaches the end position and resets to start position,
    /// the iteration count is incremented.
    pub fn next_verse(&mut self) -> Option<&Verse> {
        let pre_current_verse = *self.current_verse();
        let start_position = self.settings.bounds.start_position;
        let end_position = self.settings.bounds.end_position;
        let remaining_iterations = self.settings.bounds.iteration_limit.saturating_sub(
            self.iteration_count
        );

        // Before we move to the next verse, check if we have reached the end position
        if end_position.eq(self.current_verse()) {
            // We reached the end bound.
            if remaining_iterations > 0 {
                self.iteration_count += 1;
                self.reset_position(start_position);
                return Some(self.current_verse());
            } else {
                return None;
            }
        }

        // Try to get the next verse
        let has_verse = match self.direction {
            Direction::Downwards => self.next_verse_downward().is_ok(),
            Direction::Upwards => self.next_verse_upward().is_ok(),
        };

        if has_verse {
            return Some(self.current_verse());
        }

        None
    }

    /// Attempts to advance to the next verse within the current sura.
    ///
    /// This method tries to move forward by one verse while ensuring the navigator
    /// stays within the same sura. If the forward movement would cross into a
    /// different sura, the operation is rolled back and returns `false`.
    ///
    /// # Behavior
    /// 1. Captures current verse position
    /// 2. Attempts to move forward by one verse
    /// 3. Checks if the new verse is in the same sura
    /// 4. If different sura: rolls back and returns `false`
    /// 5. If same sura: keeps the new position and returns `true`
    ///
    /// # Returns
    /// * `true` - Successfully moved to next verse in the same sura
    /// * `false` - Movement would cross sura boundary, position unchanged
    ///
    /// # Use Cases
    /// This method is useful for navigation algorithms that need to respect
    /// sura boundaries, such as page layout calculations or verse grouping
    /// operations where crossing suras is not desired.
    fn try_advance_within_current_sura(&mut self) -> bool {
        let pre_current_verse = *self.current_verse();
        let forward_movement = self.forward_index(1).is_some();
        if forward_movement {
            if self.current_verse().sura == pre_current_verse.sura {
                return true;
            }
            // Rollback
            self.backward_index(1);
        }
        false
    }
    /// Move to the next verse in downward direction
    fn next_verse_downward(&mut self) -> Result<(), ()> {
        if self.forward_index(1).is_some() {
            return Ok(());
        }
        Err(())
    }

    /// Move to the next verse in upward direction
    fn next_verse_upward(&mut self) -> Result<(), ()> {
        if self.try_advance_within_current_sura() {
            return Ok(());
        }
        match self.move_pre_sura(self.current_verse().sura) {
            Ok(verse) => Ok(()),
            Err(e) => Err(()),
        }
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

    /// Find a verse in the mushaf and return its location (`Verse`, `page`, `index_of_verse`)
    ///
    /// # Errors
    /// * `LookupError::InvalidVerse` if the verse number is invalid
    /// * `LookupError::VerseNotFound` if the verse is not found
    ///
    /// # Panics
    /// * `expect` if `i` cannot be converted to a `u8`, where `i` is the index of the verse in the page.
    pub fn find_verse(
        &self,
        verse: impl Into<VersePosition>
    ) -> Result<(&Verse, u16, u8), LookupError> {
        let (sura_number, verse_number) = verse.into().tuple();
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

    // Index navigation
    fn move_by_index(&mut self, index: usize, backward: bool) -> Option<&Verse> {
        if self.current_page_idx >= self.mushaf.pages.len() {
            return None;
        }

        let mut page_verses = self.mushaf.pages[self.current_page_idx].verses();
        let mut remaining =
            index +
            (if backward {
                page_verses.len() - 1 - self.current_verse_idx
            } else {
                self.current_verse_idx
            });

        loop {
            if page_verses.len() > remaining {
                // Verse is in this page
                self.current_verse_idx = {
                    if backward {
                        page_verses.len().saturating_sub(remaining + 1)
                    } else {
                        remaining
                    }
                };
                return Some(&page_verses[self.current_verse_idx]);
            }

            remaining -= page_verses.len();
            // Move to next page
            self.current_page_idx = {
                match (self.current_page_idx, backward) {
                    (0, true) => {
                        return None;
                    }
                    (x, false) if x == self.mushaf.pages.len() - 1 => {
                        return None;
                    }
                    _ => {}
                }

                match backward {
                    true => { self.current_page_idx - 1 }
                    false => { self.current_page_idx + 1 }
                }
            };
            page_verses = self.mushaf.pages[self.current_page_idx].verses();
        }

        None
    }

    pub fn forward_index(&mut self, index: usize) -> Option<&Verse> {
        self.move_by_index(index, false)
    }
    pub fn backward_index(&mut self, index: usize) -> Option<&Verse> {
        self.move_by_index(index, true)
    }
}

#[cfg(test)]
mod test {
    use std::{ path::PathBuf, rc::Rc };

    use colored::Colorize;

    use crate::{
        king_fahad_mushaf::{ JsonVerse, KingFahadMushaf },
        mushaf::{ Mushaf, QuranMetadata, Verse },
        navigation::{ Direction, NavigationSettings, VersePosition, VersesNavigator },
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
        navigator.reset_position(VersePosition::new(9, 1));
        let verse = navigator.current_verse();
        let lines = navigator.calculate_verse_lines(verse);
        assert!((lines - (verse.lines + 1.0)).abs() < f32::EPSILON);

        navigator.reset_position(VersePosition::new(9, 1));
        let mut navigator = navigator.ignore_sura_header(true);
        let verse = navigator.current_verse();
        let lines = navigator.calculate_verse_lines(verse);
        assert_eq!(lines, verse.lines);

        // Loop through all first verses of suras
        navigator = navigator.ignore_sura_header(false);
        for sura in 1..=114 {
            let header_lines = if sura == 9 { 1.0 } else { 2.0 };
            navigator.reset_position(VersePosition::new(sura, 1));
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
        assert!(navigator.reset_position(VersePosition::new(5, 119)).is_ok());

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
        navigator.reset_position(VersePosition::new(5, 119));

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

        navigator.reset_position(VersePosition::new(114, 1));
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
        navigator.reset_position(VersePosition::end());

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

    // test reset iterations
    #[test]
    fn test_move_by_index_forward_same_page() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        );

        // Start at first verse of first page
        navigator.reset_position(VersePosition::new(1, 1));

        // Move forward by 2 verses within the same page
        let verse = navigator.forward_index(2);
        assert!(verse.is_some());
        let verse = verse.expect("expect `verse` not to be None");
        assert_eq!(verse.sura, 1);
        assert_eq!(verse.number, 3); // Should be 3rd verse of Al-Fatiha

        // Move forward by 1 more verse
        let verse = navigator.forward_index(1);
        assert!(verse.is_some());
        let verse = verse.expect("expect `verse` not to be None");

        assert_eq!(verse.sura, 1);
        assert_eq!(verse.number, 4);
    }

    #[test]
    fn test_move_by_index_backward_same_page() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf.clone(),
            metadata,
            Default::default(),
            Default::default()
        );
        // Start at 4th verse of first page
        navigator.reset_position(VersePosition::new(1, 4));

        // Move backward by 2 verses within the same page
        let verse = navigator.backward_index(2);
        assert!(verse.is_some());
        let verse = verse.expect("expect `verse` not to be None");
        assert_eq!(verse.sura, 1);
        assert_eq!(verse.number, 2); // Should be 2nd verse of Al-Fatiha

        // Move backward by 1 more verse
        let verse = navigator.backward_index(1);
        assert!(verse.is_some());
        let verse = verse.expect("expect `verse` not to be None");
        assert_eq!(verse.sura, 1);
        assert_eq!(verse.number, 1);
    }

    #[test]
    fn test_move_by_index_forward_cross_page() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf.clone(),
            metadata.clone(),
            Default::default(),
            Default::default()
        );

        // Start at first verse of first page
        navigator.reset_position(VersePosition::new(1, 1));

        // Get the number of verses in the first page
        let first_page_verses = mushaf.pages[0].verses();
        let first_page_verse_count = first_page_verses.len();

        let verse = navigator.forward_index(first_page_verse_count + 2);
        assert!(verse.is_some());
    }

    #[test]
    fn test_move_by_index_backward_cross_page() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf.clone(),
            metadata.clone(),
            Default::default(),
            Default::default()
        );

        // Start at a verse in the second page
        navigator.reset_position(VersePosition::new(2, 1));

        // Get the number of verses in the first page
        let first_page_verses = mushaf.pages[0].verses();
        let first_page_verse_count = first_page_verses.len();

        // Move backward by more verses than available in current page
        let verse = navigator.backward_index(first_page_verse_count + 2);
        assert!(verse.is_none());
    }

    #[test]
    fn test_move_by_index_zero_index() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        );

        navigator.reset_position(VersePosition::new(1, 1));

        // Move by 0 index should return None
        let verse = navigator.forward_index(0).copied();
        assert!(verse.is_some());
        assert_eq!(VersePosition::new(1, 1), verse.expect("expect `verse` not to be None"));

        let verse = navigator.backward_index(0).copied();
        assert!(verse.is_some());
        assert_eq!(VersePosition::new(1, 1), verse.expect("expect `verse` not to be None"));
    }

    #[test]
    fn test_move_by_index_large_index() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        );

        navigator.reset_position(VersePosition::new(1, 1));

        // Try to move by a very large index that exceeds all available verses
        let verse = navigator.forward_index(100000);
        assert!(verse.is_none());

        navigator.reset_position(VersePosition::new(1, 1));
        let verse = navigator.backward_index(100000);
        assert!(verse.is_none());
    }

    #[test]
    fn test_move_by_index_at_page_boundaries() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf.clone(),
            metadata.clone(),
            Default::default(),
            Default::default()
        );

        // Test at the last verse of a page
        let first_page_verses = mushaf.pages[0].verses();
        let last_verse_idx = first_page_verses.len() - 1;
        let last_verse = &first_page_verses[last_verse_idx];

        navigator.reset_position(VersePosition::new(last_verse.sura, last_verse.number));

        // Move forward by 1 should go to next page
        let verse = navigator.forward_index(1);
        assert!(verse.is_some());
        let verse = verse.expect("expect `verse` not to be None");
        // Should be in the second page
        assert!(mushaf.pages[1].verses().contains(verse));

        // Test at the first verse of a page
        navigator.reset_position(VersePosition::new(2, 1));

        // Move backward by 1 should go to previous page
        let verse = navigator.backward_index(1);
        assert!(verse.is_some());
        let verse = verse.expect("expect `verse` not to be None");
        // Should be in the first page
        assert!(mushaf.pages[0].verses().contains(verse));
    }

    #[test]
    fn test_move_by_index_consistency_with_sequential_navigation() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf.clone(),
            metadata.clone(),
            Default::default(),
            Default::default()
        );

        navigator.reset_position(VersePosition::new(1, 1));

        // Move forward by 3 using index navigation
        let verse_by_index = navigator.forward_index(3).copied();
        assert!(verse_by_index.is_some());
        let verse_by_index = verse_by_index.expect("expect `verse_by_index` not to be None");

        // Reset and move forward 3 times using sequential navigation
        navigator.reset_position(VersePosition::new(1, 1));
        let mut verse_by_sequential = navigator.current_verse();
        for _ in 0..3 {
            verse_by_sequential = navigator
                .next_verse()
                .expect("expect `verse_by_sequential` not to be None");
        }

        // Results should be the same
        assert_eq!(verse_by_index.sura, verse_by_sequential.sura);
        assert_eq!(verse_by_index.number, verse_by_sequential.number);
    }

    #[test]
    fn test_move_by_index_with_different_directions() {
        let (mushaf, metadata) = get_mushaf();

        // Test with Upwards direction
        let mut navigator_up = VersesNavigator::new(
            Rc::clone(&mushaf),
            Rc::clone(&metadata),
            Default::default(),
            Direction::Upwards
        );

        navigator_up.reset_position(VersePosition::new(2, 5));
        let verse_up = navigator_up.forward_index(3);
        assert!(verse_up.is_some());

        // Test with Downwards direction
        let mut navigator_down = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Direction::Downwards
        );

        navigator_down.reset_position(VersePosition::new(2, 5));
        let verse_down = navigator_down.forward_index(3);
        assert!(verse_down.is_some());

        // Both should return valid verses (though potentially different due to direction)
        assert!(verse_up.expect("expect `verse_up` not to be None").sura > 0);
        assert!(verse_down.expect("expect `verse_down` not to be None").sura > 0);
    }

    #[test]
    fn test_move_by_index_invalid_page_index() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        );

        // Set an invalid page index (beyond available pages)
        navigator.current_page_idx = 1000;

        // Any movement should return None
        let verse = navigator.forward_index(1);
        assert!(verse.is_none());

        let verse = navigator.backward_index(1);
        assert!(verse.is_none());
    }

    #[test]
    fn test_reset_iterations() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        );

        navigator.reset_iterations(2);
        assert_eq!(navigator.iteration_count, 2);

        navigator.reset_iterations(10);
        assert_eq!(navigator.iteration_count, 10);
    }

    #[test]
    fn test_reverse_bounds_manually() {
        let settings = NavigationSettings::builder()
            .start_position(VersePosition::new(2, 1))
            .end_position(VersePosition::new(2, 286))
            .reverse_bounds();
        assert_eq!(settings.bounds.start_position, VersePosition::new(2, 286));
        assert_eq!(settings.bounds.end_position, VersePosition::new(2, 1));
    }

    #[test]
    fn test_reverse_bounds_automatically_with_direction_changes() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Direction::Downwards
        );

        let start_position = navigator.settings.bounds.start_position;
        let end_position = navigator.settings.bounds.end_position;

        navigator = navigator.direction(Direction::Upwards);
        assert_eq!(navigator.settings.bounds.start_position, end_position);
        assert_eq!(navigator.settings.bounds.end_position, start_position);
    }

    #[test]
    fn test_reset_position() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        );
        navigator.reset_position(VersePosition::new(2, 1));
        assert_eq!(navigator.current_verse().sura, 2);
        assert_eq!(navigator.current_verse().number, 1);

        navigator.reset_position(VersePosition::new(2, 286));
        assert_eq!(navigator.current_verse().sura, 2);
        assert_eq!(navigator.current_verse().number, 286);

        navigator.reset_position(VersePosition::end());
        assert_eq!(navigator.current_verse().sura, 114);
        assert_eq!(navigator.current_verse().number, 6);
    }

    #[test]
    fn test_reset_position_invalid_verse() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        );

        assert!(navigator.reset_position(VersePosition::new(2, 287)).is_err());
        assert!(navigator.reset_position(VersePosition::new(2, 0)).is_err());
        assert!(navigator.reset_position(VersePosition::new(114, 7)).is_err());
        assert!(navigator.reset_position(VersePosition::new(114, 0)).is_err());
        assert!(navigator.reset_position(VersePosition::new(112, 90)).is_err());
    }

    #[test]
    fn test_multiple_iterations() {
        let (mushaf, metadata) = get_mushaf();
        let mut navigator = VersesNavigator::new(
            mushaf,
            metadata,
            Default::default(),
            Default::default()
        )
            .start_position(VersePosition::new(2, 1))
            .end_position(VersePosition::new(2, 286))
            .iteration_limit(1);
        navigator.reset_position(VersePosition::new(2, 1));

        let sura_2_lines = {
            let (mushaf, metadata) = get_mushaf();
            let mut nav = VersesNavigator::new(
                mushaf,
                metadata,
                Default::default(),
                Default::default()
            );
            nav.reset_position(VersePosition::new(2, 1));
            let mut lines = 0.0;
            println!("===== Start of `sura_2_lines` =====");
            loop {
                let verse = nav.current_verse();
                if verse.sura != 2 {
                    break;
                }
                if verse.number == 1 || verse.number == 286 {
                    println!("{verse}");
                } else {
                    print!(".");
                }
                lines += verse.lines;

                nav.next_verse();
            }
            println!("===== End of `sura_2_lines` =====");
            lines
        };
        assert_eq!(navigator.iteration_count, 0);

        let mut lines_count = 0.0;
        println!("===== Start of `lines_count` =====");
        loop {
            let verse = navigator.current_verse();
            if verse.number == 1 || verse.number == 286 {
                println!("{verse}");
            } else {
                print!(".");
            }
            if verse.sura != 2 {
                unreachable!();
            }
            lines_count += verse.lines;

            if navigator.next_verse().is_none() {
                break;
            }
        }
        println!("===== End of `lines_count` =====");
        assert_eq!(navigator.iteration_count + 1, 2);
        assert!((sura_2_lines * 2.0 - lines_count).abs() < f32::EPSILON);
    }
}
