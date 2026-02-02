use crate::{
    mushaf::{Mushaf, QuranMetadata, Verse},
    navigation::{LookupError, NavigationSettings, VersePosition},
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
/// - When reaching `lower_bound` with remaining iterations: resets to `upper_bound`
/// - When reaching `lower_bound` with no remaining iterations: returns `None`
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
/// 1. Navigator starts at `upper_bound`
/// 2. Moves through verses until reaching `lower_bound`
/// 3. If `remaining_iterations > 0`: increment count and reset to `upper_bound`
/// 4. If `remaining_iterations = 0`: return `None` (stop navigation)
///
/// ## Practical Examples
///
/// ### Reading Al-Fatiha Three Times
/// ```ignore
/// use rust_quran_engine::navigation::{VersesNavigator, VersePosition, Direction};
/// let navigator = VersesNavigator::builder(mushaf, metadata)
///     .upper_bound(VersePosition::new(1, 1))
///     .lower_bound(VersePosition::new(1, 7))
///     .iteration_limit(3)  // Read Al-Fatiha 3 times
///     .direction(Direction::Downwards);
/// ```
///
/// ### Page Layout Calculation
/// ```ignore
/// use rust_quran_engine::navigation::{VersesNavigator, VersePosition};
/// let navigator = VersesNavigator::builder(mushaf, metadata)
///     .upper_bound(VersePosition::new(2, 1))
///     .lower_bound(VersePosition::new(2, 50))
///     .iteration_limit(0)  // No cycling for layout
///     .ignore_sura_header(true);  // Exclude headers for precise calculation
/// ```
///
/// ### Memorization Practice
/// ```ignore
/// use rust_quran_engine::navigation::{VersesNavigator, VersePosition, Direction};
/// let navigator = VersesNavigator::builder(mushaf, metadata)
///     .upper_bound(VersePosition::new(67, 1))
///     .lower_bound(VersePosition::new(67, 30))
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
    ///     .upper_bound(VersePosition::new(1, 1))
    ///     .lower_bound(VersePosition::new(1, 7))
    ///     .iteration_limit(2);
    ///
    /// ```
    /// ```ignore
    /// use rust_quran_engine::navigation::{VersesNavigator, Direction};
    /// let navigator = VersesNavigator::new(mushaf, metadata, settings, Direction::Downwards);
    /// ```
    #[must_use]
    pub fn new(
        mushaf: Rc<Mushaf>,
        metadata: Rc<QuranMetadata>,
        settings: NavigationSettings,
        direction: Direction,
    ) -> Self {
        let mut s = Self {
            mushaf,
            quran_metadata: metadata,
            current_page_idx: 0,
            current_verse_idx: 0,
            direction,
            settings,
            iteration_count: 0,
        };

        s.reset_position(s.get_start_bound())
            .expect("expect `reset_position` to succeed");
        s
    }

    /// Creates a builder for `VersesNavigator` with default settings.
    ///
    /// Default settings:
    /// - `direction`: `Direction::Downwards`
    /// - `iteration_limit`: 0 (no cycling)
    /// - `upper_bound`: (1, 1) - beginning of Al-Fatiha
    /// - `lower_bound`: (114, 6) - end of An-Nas
    /// - `ignore_sura_header`: false (include headers)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_quran_engine::navigation::{VersesNavigator, VersePosition, Direction};
    /// let navigator = VersesNavigator::builder(mushaf, metadata)
    ///     .upper_bound(VersePosition::new(2, 1))
    ///     .lower_bound(VersePosition::new(2, 50))
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
        direction: Direction,
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
    /// let navigator = navigator.direction(Direction::Upwards); // Bounds remain consistent
    /// ```
    #[must_use]
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
    /// Helper to get the start bound based on the direction & bounds mode
    pub fn get_start_bound(&self) -> VersePosition {
        // In excluding mode, bounds are full Quran, not the configured bounds
        if self.settings.bounds.is_excluding_mode() {
            match self.direction {
                Direction::Downwards => VersePosition::start(), // (1,1)
                Direction::Upwards => {
                    // Upward navigation reads suras in reverse order (114→1),
                    // starting at verse 1 of each sura. So start at (114, 1).
                    VersePosition::new(114, 1)
                }
            }
        } else {
            match self.direction {
                Direction::Downwards => self.settings.bounds.upper_bound,
                Direction::Upwards => {
                    // Start at lower_bound for upward navigation.
                    // Navigation reads forward within sura, then moves to previous suras.
                    // E.g., for range (6:150) to (15:9), upward starts at 15:9,
                    // reads 15:9→15:99, then 14:1→14:end, ..., ending at 6:150.
                    self.settings.bounds.lower_bound
                }
            }
        }
    }

    /// Helper to get the end bound based on the direction & bounds mode
    pub fn get_end_bound(&self) -> VersePosition {
        let b = self.settings.bounds;

        // In excluding mode, bounds are full Quran, not the configured bounds
        if b.is_excluding_mode() {
            match self.direction {
                Direction::Downwards => VersePosition::end(), // (114,6)
                Direction::Upwards => {
                    // Upward navigation reads suras in reverse order (114→1),
                    // ending at the last verse of sura 1. Sura 1 (Al-Fatiha) has 7 verses.
                    let total_verses = self
                        .quran_metadata
                        .get_sura_info(1)
                        .expect("expect `sura_info` not to be None for sura 1")
                        .total_verses;
                    VersePosition::new(1, total_verses) // (1, 7)
                }
            }
        } else {
            match self.direction {
                Direction::Downwards => b.lower_bound,
                Direction::Upwards => {
                    if b.upper_bound.verse() == 1 {
                        // upper_bound starts at verse 1, so navigate to end of that sura
                        // (or lower_bound if end of sura is out of bounds)
                        let total_verses = self
                            .quran_metadata
                            .get_sura_info(b.upper_bound.sura())
                            .expect("expect `sura_info` not to be None")
                            .total_verses;
                        let end_of_sura = VersePosition::new(b.upper_bound.sura(), total_verses);
                        if self.is_out_of_bounds(end_of_sura) {
                            b.lower_bound
                        } else {
                            end_of_sura
                        }
                    } else if b.upper_bound.sura() == b.lower_bound.sura() {
                        // Same sura, partial range: end at lower_bound
                        b.lower_bound
                    } else {
                        // Multi-sura with specific upper_bound verse: stop exactly at upper_bound
                        b.upper_bound
                    }
                }
            }
        }
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

    /// Sets the upper bound for navigation bounds.
    ///
    /// # Arguments
    /// * `upper_bound` - Upper bound for navigation
    ///
    /// # Bounds Behavior
    /// - When `upper_bound < lower_bound`: Navigate from upper_bound to lower_bound (inclusive)
    /// - When `upper_bound > lower_bound`: Navigate on the whole range EXCLUDING the range from lower_bound to upper_bound (inclusive)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Normal inclusive mode
    /// let navigator = navigator.upper_bound(VersePosition::new(1, 1));
    ///
    /// // Excluding mode - navigate everywhere except verses 1-50 of Sura 2
    /// let navigator = navigator
    ///     .upper_bound(VersePosition::new(2, 50))
    ///     .lower_bound(VersePosition::new(2, 1));
    /// ```
    #[must_use]
    pub fn upper_bound(mut self, upper_bound: impl Into<VersePosition>) -> Self {
        self.settings.bounds.upper_bound = upper_bound.into();
        self
    }

    /// Sets the lower bound for navigation bounds.
    ///
    /// # Arguments
    /// * `lower_bound` - Lower bound for navigation
    ///
    /// # Bounds Behavior
    /// - When `upper_bound < lower_bound`: Navigate from upper_bound to lower_bound (inclusive)
    /// - When `upper_bound > lower_bound`: Navigate on the whole range EXCLUDING the range from lower_bound to upper_bound (inclusive)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Normal inclusive mode
    /// let navigator = navigator.lower_bound(VersePosition::new(114, 6));
    ///
    /// // Excluding mode - navigate everywhere except verses 1-50 of Sura 2
    /// let navigator = navigator
    ///     .upper_bound(VersePosition::new(2, 50))
    ///     .lower_bound(VersePosition::new(2, 1));
    /// ```
    #[must_use]
    pub fn lower_bound(mut self, lower_bound: impl Into<VersePosition>) -> Self {
        self.settings.bounds.lower_bound = lower_bound.into();
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
        verse: impl Into<VersePosition>,
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
        let upper_bound = self.settings.bounds.upper_bound;
        let lower_bound = self.settings.bounds.lower_bound;

        // Check if we're in excluding mode
        if self.settings.bounds.is_excluding_mode() {
            // In excluding mode: verse is out of bounds if it's within the excluded range
            // (from lower_bound to upper_bound, exclusive)
            let verse_pos: VersePosition = verse.into();
            return verse_pos > lower_bound && verse_pos < upper_bound;
        }

        // Normal inclusive mode: verse is out of bounds if it's outside the range
        let is_out_of_bounds_sura: bool =
            verse_sura < upper_bound.sura() || verse_sura > lower_bound.sura();
        if is_out_of_bounds_sura {
            return true;
        }

        // Verse sura in the bounds

        // If both bounds are in the same sura, check both conditions
        if upper_bound.sura() == lower_bound.sura() && verse_sura == upper_bound.sura() {
            return verse_number < upper_bound.verse() || verse_number > lower_bound.verse();
        }

        if verse_sura == upper_bound.sura() {
            return verse_number < upper_bound.verse();
        }
        if verse_sura == lower_bound.sura() {
            return verse_number > lower_bound.verse();
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
    ///     .upper_bound(VersePosition::new(1, 1))
    ///     .lower_bound(VersePosition::new(1, 7))
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
    ///     .upper_bound(VersePosition::new(1, 1))
    ///     .lower_bound(VersePosition::new(1, 7))
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
    ///     .upper_bound(VersePosition::new(2, 1))
    ///     .lower_bound(VersePosition::new(2, 5))
    ///     .direction(Direction::Upwards); // Navigate backwards
    ///
    /// // Will navigate from (2,1) to (2,5) with bounds remaining consistent
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
        let start_bound = self.get_start_bound();
        let end_bound = self.get_end_bound();
        let remaining_iterations =
            self.settings.bounds.iteration_limit.saturating_sub(self.iteration_count);

        // Before we move to the next verse, check if we have reached the lower bound
        if end_bound.eq(self.current_verse()) {
            // Either end of quran, or the lower bound. We need to reset the position, or to
            // stop the navigation if we ran out of iterations.
            if remaining_iterations > 0 {
                self.iteration_count += 1;
                let _ = self.reset_position(start_bound);
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
            // In excluding mode, skip over excluded verses
            if self.settings.bounds.is_excluding_mode() {
                loop {
                    let current = self.current_verse();

                    if !self.is_out_of_bounds(current.into()) {
                        break;
                    }

                    let moved = match self.direction {
                        Direction::Downwards => self.next_verse_downward().is_ok(),
                        Direction::Upwards => self.next_verse_upward().is_ok(),
                    };
                    if !moved {
                        return None;
                    }
                }
            }
            return Some(self.current_verse());
        }

        None
    }

    /// Moves to the previous verse based on direction and iteration bounds.
    ///
    /// This is the inverse of the `next_verse()` method, allowing backward navigation
    /// through verses while respecting iteration limits and bounded ranges.
    ///
    /// # Iteration Behavior
    ///
    /// The method implements the following iteration logic:
    /// 1. Check if current position equals the `start_position`
    /// 2. If at start position and iterations remain: increment count and reset to `end_position`
    /// 3. If at start position and no iterations remain: return `None` (stop navigation)
    /// 4. Otherwise: move to previous verse in the specified direction
    ///
    /// # Return Value
    /// * `Some(&Verse)` - Successfully moved to previous verse
    /// * `None` - Reached start of bounds with no remaining iterations
    ///
    /// # Examples
    ///
    /// ## Basic Backward Navigation
    /// ```ignore
    /// let mut navigator = VersesNavigator::builder(mushaf, metadata)
    ///     .upper_bound(VersePosition::new(1, 1))
    ///     .lower_bound(VersePosition::new(1, 7))
    ///     .iteration_limit(0); // No cycling
    ///
    /// // Start at end of Al-Fatiha
    /// navigator.reset_position(VersePosition::new(1, 7));
    ///
    /// // Navigate backward through Al-Fatiha
    /// while let Some(verse) = navigator.previous_verse() {
    ///     println!("{}", verse);
    /// }
    /// ```
    ///
    /// ## Direction-Aware Backward Navigation
    /// ```ignore
    /// let mut navigator = VersesNavigator::builder(mushaf, metadata)
    ///     .upper_bound(VersePosition::new(2, 1))
    ///     .lower_bound(VersePosition::new(2, 5))
    ///     .direction(Direction::Upwards); // Navigate backwards
    ///
    /// // Navigate backward from (2,5) to (2,1)
    /// while let Some(verse) = navigator.previous_verse() {
    ///     println!("{}", verse);
    /// }
    /// ```
    pub fn previous_verse(&mut self) -> Option<&Verse> {
        let start_bound = self.get_start_bound();
        let end_bound = self.get_end_bound();
        let remaining_iterations =
            self.settings.bounds.iteration_limit.saturating_sub(self.iteration_count);

        // Before moving, check if we're at the start bound
        if start_bound.eq(self.current_verse()) {
            if remaining_iterations > 0 {
                self.iteration_count += 1;
                let _ = self.reset_position(end_bound);
                return Some(self.current_verse());
            } else {
                return None;
            }
        }

        // Move to previous verse based on direction
        let has_verse = match self.direction {
            Direction::Downwards => self.prev_verse_downward().is_ok(),
            Direction::Upwards => self.prev_verse_upward().is_ok(),
        };

        if has_verse {
            // In excluding mode, skip over excluded verses
            if self.settings.bounds.is_excluding_mode() {
                loop {
                    let current = self.current_verse();

                    if !self.is_out_of_bounds(current.into()) {
                        break;
                    }

                    let moved = match self.direction {
                        Direction::Downwards => self.prev_verse_downward().is_ok(),
                        Direction::Upwards => self.prev_verse_upward().is_ok(),
                    };
                    if !moved {
                        return None;
                    }
                }
            }
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
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn move_pre_sura(&mut self, current_sura: u8) -> Result<&Verse, LookupError> {
        // When at sura 1 in upward navigation, wrap to sura 114
        let target_sura = if current_sura == 1 {
            114
        } else {
            current_sura - 1
        };

        let pre_sura = self.quran_metadata.get_sura_info(target_sura)?;
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

    /// Move to the previous verse in downward direction
    fn prev_verse_downward(&mut self) -> Result<(), ()> {
        if self.backward_index(1).is_some() {
            return Ok(());
        }
        Err(())
    }

    /// Attempts to retreat to the previous verse within the current sura.
    ///
    /// This method tries to move backward by one verse while ensuring the navigator
    /// stays within the same sura. If the backward movement would cross into a
    /// different sura, the operation is rolled back and returns `false`.
    ///
    /// # Behavior
    /// 1. Captures current verse position
    /// 2. Attempts to move backward by one verse
    /// 3. Checks if the new verse is in the same sura
    /// 4. If different sura: rolls back and returns `false`
    /// 5. If same sura: keeps the new position and returns `true`
    ///
    /// # Returns
    /// * `true` - Successfully moved to previous verse in the same sura
    /// * `false` - Movement would cross sura boundary, position unchanged
    fn try_retreat_within_current_sura(&mut self) -> bool {
        let pre_current_verse = *self.current_verse();
        let backward_movement = self.backward_index(1).is_some();
        if backward_movement {
            if self.current_verse().sura == pre_current_verse.sura {
                return true;
            }
            // Rollback
            self.forward_index(1);
        }
        false
    }

    fn move_next_sura(&mut self, current_sura: u8) -> Result<&Verse, LookupError> {
        // When at sura 114 in upward navigation's previous, wrap to sura 1
        let target_sura = if current_sura == 114 {
            1
        } else {
            current_sura + 1
        };

        let next_sura = self.quran_metadata.get_sura_info(target_sura)?;
        // For upward direction's "previous", go to end of next sura
        let last_verse_position = VersePosition::new(next_sura.number, next_sura.total_verses);
        let (_, page, idx) = self.find_verse(last_verse_position)?;
        self.current_page_idx = (page as usize) - 1;
        self.current_verse_idx = idx as usize;
        Ok(self.current_verse())
    }

    /// Move to the previous verse in upward direction
    fn prev_verse_upward(&mut self) -> Result<(), ()> {
        // Try to go backward within current sura
        if self.try_retreat_within_current_sura() {
            return Ok(());
        }
        // Move to the next sura (in upward navigation, "previous" means higher sura number)
        match self.move_next_sura(self.current_verse().sura) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
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
        verse: impl Into<VersePosition>,
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
            return (total_lines * 10.0).round() / 10.0;
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

        (total_lines * 10.0).round() / 10.0
    }

    // Index navigation
    fn move_by_index(&mut self, index: usize, backward: bool) -> Option<&Verse> {
        if self.current_page_idx >= self.mushaf.pages.len() {
            return None;
        }

        let mut page_verses = self.mushaf.pages[self.current_page_idx].verses();
        let mut remaining_verses = self.get_remaining_verses(index, backward, page_verses);

        loop {
            if page_verses.len() > remaining_verses {
                self.current_verse_idx = {
                    if backward {
                        page_verses.len().saturating_sub(remaining_verses + 1)
                    } else {
                        remaining_verses
                    }
                };
                return Some(&page_verses[self.current_verse_idx]);
            }

            remaining_verses -= page_verses.len();
            match (self.current_page_idx, backward) {
                (0, true) => {
                    return None;
                }
                (x, false) if x == self.mushaf.pages.len() - 1 => {
                    return None;
                }
                _ => {}
            }
            // Move to next page
            self.current_page_idx = {
                match backward {
                    true => self.current_page_idx - 1,
                    false => self.current_page_idx + 1,
                }
            };
            page_verses = self.mushaf.pages[self.current_page_idx].verses();
        }
    }

    fn get_remaining_verses(&self, index: usize, backward: bool, page_verses: &[Verse]) -> usize {
        if backward {
            page_verses.len() - 1 - self.current_verse_idx + index
        } else {
            self.current_verse_idx + index
        }
    }

    pub fn forward_index(&mut self, index: usize) -> Option<&Verse> {
        self.move_by_index(index, false)
    }
    pub fn backward_index(&mut self, index: usize) -> Option<&Verse> {
        self.move_by_index(index, true)
    }
}

#[cfg(test)]
#[path = "verse_navigator_tests.rs"]
mod tests;
