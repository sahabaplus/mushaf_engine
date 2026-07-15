use super::{LastVerseResult, OverflowResult};
use crate::mushaf::Verse;

#[cfg(feature = "colored_output")]
use colored::Colorize;

/// Information about navigation cycles and boundary crossings
///
/// This structure groups semantically related cycle information to improve
/// code organization and reduce function parameter count.
///
/// Note: `Eq` is not derived because `cycle_distance` is an `f32`, which does not implement `Eq`
/// due to NaN handling. Use `PartialEq` for equality comparisons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CycleInfo {
    /// Number of times navigation passed through the starting verse again
    ///
    /// This counts how many times the navigator returned to the initial starting verse
    /// during navigation. For example, if navigating from (10,1) by 10000 lines ends at
    /// (10,100), a `cycles_completed` of 2 means we passed through (10,1) twice during
    /// navigation, explaining why the large distance resulted in a nearby verse.
    ///
    /// This is essential for `calculate_lines` to accurately compute the distance when
    /// given a start verse, end verse, and cycle count.
    cycles_completed: u32,
    /// Whether the navigation crossed boundaries (passed starting point or wrapped mushaf)
    ///
    /// This is true when:
    /// - Navigation passed through the starting verse at least once (`cycles_completed > 0`)
    /// - Mushaf wrapping occurred (sura 1 → 114 upward, or 114 → 1 downward)
    ///
    /// This helps callers understand non-linear navigation paths where the end position
    /// may appear confusing relative to the start position.
    crossed_boundaries: bool,
    /// The distance (in lines) of one full cycle through the bounded region
    ///
    /// This is the number of lines required to navigate from the starting verse back to itself
    /// when bounded navigation is enabled. For example, if navigating (2,1) → (2,50) → (2,1)
    /// in a bounded region takes 150 lines, `cycle_distance` would be 150.0.
    ///
    /// This field is 0.0 when:
    /// - No cycles were completed (`cycles_completed == 0`)
    /// - No bounds are configured (unbounded navigation)
    ///
    /// This enables callers to compute total distance using the formula:
    /// ```text
    /// total_distance = (cycles_completed - 1) * cycle_distance + direct_distance
    /// ```
    ///
    /// where `direct_distance` can be calculated using `calculate_lines(start, end, direction, settings)`.
    ///
    /// This formula works because:
    /// - `cycles_completed` counts passes through the start verse
    /// - Each pass adds one `cycle_distance` worth of lines
    /// - The current (partial or complete) cycle contributes `direct_distance`
    /// - So we have (N-1) complete previous cycles + current cycle
    cycle_distance: f32,
}

impl Default for CycleInfo {
    fn default() -> Self {
        Self {
            cycles_completed: 0,
            crossed_boundaries: false,
            cycle_distance: 0.0,
        }
    }
}

impl CycleInfo {
    /// Create a new `CycleInfo` with the specified cycle tracking data
    ///
    /// # Arguments
    /// * `cycles_completed` - Number of cycles through the starting verse
    /// * `crossed_boundaries` - Whether navigation wrapped around boundaries
    /// * `cycle_distance` - Distance in lines for one full cycle
    #[must_use]
    pub const fn new(cycles_completed: u32, crossed_boundaries: bool, cycle_distance: f32) -> Self {
        Self {
            cycles_completed,
            crossed_boundaries,
            cycle_distance,
        }
    }

    /// Create a new `CycleInfo` with no cycles
    ///
    /// Use this for const contexts. For regular code, prefer `CycleInfo::default()`.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            cycles_completed: 0,
            crossed_boundaries: false,
            cycle_distance: 0.0,
        }
    }

    /// Get the number of cycles completed
    #[must_use]
    pub const fn cycles_completed(&self) -> u32 {
        self.cycles_completed
    }

    /// Get whether boundaries were crossed
    #[must_use]
    pub const fn crossed_boundaries(&self) -> bool {
        self.crossed_boundaries
    }

    /// Get the cycle distance
    #[must_use]
    pub const fn cycle_distance(&self) -> f32 {
        self.cycle_distance
    }

    /// Reconstruct total navigated distance from direct distance
    ///
    /// This helper method implements the formula:
    /// ```text
    /// total_distance = (cycles_completed - 1) * cycle_distance + direct_distance
    /// ```
    ///
    /// # Arguments
    /// * `direct_distance` - The direct distance between start and end verses
    ///
    /// # Returns
    /// The total distance including all complete cycles
    ///
    /// # Example
    /// ```
    /// # use mushaf_engine::navigation::CycleInfo;
    /// let cycle_info = CycleInfo::new(3, true, 100.0);
    /// let direct_distance = 25.0;
    /// let total = cycle_info.reconstruct_distance(direct_distance);
    /// assert_eq!(total, 225.0); // (3-1)*100 + 25
    /// ```
    #[must_use]
    pub const fn reconstruct_distance(&self, direct_distance: f32) -> f32 {
        if self.cycles_completed > 0 {
            ((self.cycles_completed - 1) as f32) * self.cycle_distance + direct_distance
        } else {
            direct_distance
        }
    }
}

/// Comprehensive result of a navigation operation in the Quran
///
/// This structure contains the primary verse reached through navigation,
/// as well as optional information about any overflow conditions or boundary
/// verses (last verse of page or sura) encountered during navigation.
#[derive(Debug, Clone)]
pub struct NavigationResult {
    /// Reference to the verse reached through navigation
    pub verse: Verse,
    /// Optional information about overflow if navigation exceeded boundaries
    pub overflow: Option<OverflowResult>,
    /// Optional information about the last verse of the page if encountered during navigation
    pub end_of_page: Option<LastVerseResult>,
    /// Optional information about the last verse of the sura if encountered during navigation
    pub end_of_sura: Option<LastVerseResult>,
    /// The actual distance moved during navigation in lines
    pub distance_moved: f32,
    /// The remaining distance that could not be navigated due to boundaries
    pub remaining_distance: f32,
    /// Information about navigation cycles and boundary crossings
    pub cycle_info: CycleInfo,
}

impl NavigationResult {
    /// Create a new `NavigationResult` with all possible parameters
    ///
    /// # Arguments
    /// * `verse` - Reference to the verse reached through navigation
    /// * `overflow` - Optional information about overflow if navigation exceeded boundaries
    /// * `last_of_page` - Optional information about the last verse of the page if encountered
    /// * `last_of_sura` - Optional information about the last verse of the sura if encountered
    /// * `distance_moved` - The actual distance moved during navigation in lines
    /// * `cycle_info` - Information about navigation cycles and boundary crossings
    ///
    /// # Returns
    /// A new `NavigationResult` instance with all specified information
    #[must_use]
    pub fn new(
        verse: Verse,
        overflow: Option<OverflowResult>,
        end_of_page: Option<LastVerseResult>,
        end_of_sura: Option<LastVerseResult>,
        distance_moved: f32,
        cycle_info: CycleInfo,
    ) -> Self {
        let remaining_distance =
            overflow.as_ref().map_or(0.0, |v| v.overflowed_verse.lines - v.overflow_lines);
        Self {
            verse,
            remaining_distance: (remaining_distance * 100.0).round() / 100.0,
            overflow,
            end_of_page,
            end_of_sura,
            distance_moved,
            cycle_info,
        }
    }

    /// Create a new `NavigationResult` for a normal navigation with no boundary conditions
    ///
    /// # Arguments
    /// * `verse` - Reference to the verse reached through navigation
    /// * `distance_moved` - The actual distance moved during navigation in lines
    ///
    /// # Returns
    /// A new `NavigationResult` instance with only the target verse and no boundary information
    #[must_use]
    pub const fn new_normal(verse: Verse, distance_moved: f32) -> Self {
        Self {
            verse,
            overflow: None,
            end_of_page: None,
            end_of_sura: None,
            distance_moved,
            remaining_distance: 0.0,
            cycle_info: CycleInfo::none(),
        }
    }

    /// Create a new `NavigationResult` for a navigation that includes overflow
    ///
    /// # Arguments
    /// * `verse` - Reference to the verse reached through navigation
    /// * `overflow` - Optional information about overflow if navigation exceeded boundaries
    /// * `distance_moved` - The actual distance moved during navigation in lines
    ///
    /// # Returns
    /// A new `NavigationResult` instance with the target verse and overflow information
    #[must_use]
    pub fn new_overflowed(verse: Verse, overflow: OverflowResult, distance_moved: f32) -> Self {
        let remaining_distance = overflow.overflowed_verse.lines - overflow.overflow_lines;
        Self {
            verse,
            remaining_distance: (remaining_distance * 100.0).round() / 100.0,
            overflow: Some(overflow),
            end_of_page: None,
            end_of_sura: None,
            distance_moved,
            cycle_info: CycleInfo::default(),
        }
    }

    /// Check if this navigation result encountered any boundaries
    ///
    /// # Returns
    /// `true` if the navigation hit the last verse of page, sura, or had an overflow
    #[must_use]
    pub const fn has_boundaries(&self) -> bool {
        self.overflow.is_some() || self.end_of_page.is_some() || self.end_of_sura.is_some()
    }

    /// Check if this navigation result encountered an overflow condition
    ///
    /// # Returns
    /// `true` if the navigation exceeded available boundaries
    #[must_use]
    pub const fn has_overflow(&self) -> bool {
        self.overflow.is_some()
    }

    /// Get the total overflow lines if any exist
    ///
    /// # Returns
    /// The number of overflow lines, or 0.0 if no overflow occurred
    #[must_use]
    pub const fn overflow_lines(&self) -> f32 {
        match &self.overflow {
            Some(overflow) => overflow.overflow_lines,
            None => 0.0,
        }
    }

    /// Create a new `NavigationResult` that reached the last verse of a page
    ///
    /// # Arguments
    /// * `verse` - Reference to the verse reached through navigation
    /// * `last_of_page` - Information about the last verse of the page
    /// * `distance_moved` - The actual distance moved during navigation in lines
    ///
    /// # Returns
    /// A new `NavigationResult` instance with the target verse and last-of-page information
    #[must_use]
    pub const fn new_page_boundary(
        verse: Verse,
        last_of_page: LastVerseResult,
        distance_moved: f32,
    ) -> Self {
        Self {
            verse,
            overflow: None,
            end_of_page: Some(last_of_page),
            end_of_sura: None,
            distance_moved,
            remaining_distance: 0.0,
            cycle_info: CycleInfo::none(),
        }
    }

    /// Create a new `NavigationResult` that reached the last verse of a sura
    ///
    /// # Arguments
    /// * `verse` - Reference to the verse reached through navigation
    /// * `last_of_sura` - Information about the last verse of the sura
    /// * `distance_moved` - The actual distance moved during navigation in lines
    ///
    /// # Returns
    /// A new `NavigationResult` instance with the target verse and last-of-sura information
    #[must_use]
    pub const fn new_sura_boundary(
        verse: Verse,
        last_of_sura: LastVerseResult,
        distance_moved: f32,
    ) -> Self {
        Self {
            verse,
            overflow: None,
            end_of_page: None,
            end_of_sura: Some(last_of_sura),
            distance_moved,
            remaining_distance: 0.0,
            cycle_info: CycleInfo::none(),
        }
    }
}

// Add Display implementation with conditional colored output
impl std::fmt::Display for NavigationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[cfg(feature = "colored_output")]
        {
            // Start with the primary verse information
            writeln!(f, "{:16} {}", "Target verse:".blue().bold(), self.verse)?;
            writeln!(
                f,
                "{:16} {}",
                "Distance Moved:".blue().bold(),
                self.distance_moved.to_string().yellow().bold()
            )?;
            writeln!(
                f,
                "{:16} {}",
                "Remaining Dis:".blue().bold(),
                self.remaining_distance.to_string().yellow().bold()
            )?;

            // Add overflow information if present
            if let Some(overflow) = &self.overflow {
                writeln!(f, "{:16} {}", "Overflow:".yellow().bold(), overflow)?;
            }

            // Add end of page information if present
            if let Some(end_page) = &self.end_of_page {
                writeln!(f, "{:16} {}", "End of page:".cyan().bold(), end_page)?;
            }

            // Add end of sura information if present
            if let Some(end_sura) = &self.end_of_sura {
                writeln!(f, "{:16} {}", "End of sura:".magenta().bold(), end_sura)?;
            }

            // Add cycles information if any cycles were completed
            if self.cycle_info.cycles_completed() > 0 {
                writeln!(
                    f,
                    "{:16} {}",
                    "Cycles:".green().bold(),
                    self.cycle_info.cycles_completed().to_string().yellow().bold()
                )?;
            }

            // Add cycle distance if any cycles were tracked
            if self.cycle_info.cycle_distance() > 0.0 {
                writeln!(
                    f,
                    "{:16} {}",
                    "Cycle Distance:".green().bold(),
                    self.cycle_info.cycle_distance().to_string().yellow().bold()
                )?;
            }

            // Add boundary crossing information if applicable
            if self.cycle_info.crossed_boundaries() {
                writeln!(
                    f,
                    "{:16} {}",
                    "Crossed Bounds:".red().bold(),
                    "Yes (wrapped around)".yellow().bold()
                )?;
            }

            Ok(())
        }

        #[cfg(not(feature = "colored_output"))]
        {
            // Start with the primary verse information
            writeln!(f, "Target verse: {}", self.verse)?;

            // Add overflow information if present
            if let Some(overflow) = &self.overflow {
                writeln!(f, "Overflow: {}", overflow)?;
            }

            // Add end of page information if present
            if let Some(end_page) = &self.end_of_page {
                writeln!(f, "Last of page: {}", end_page)?;
            }

            // Add end of sura information if present
            if let Some(end_sura) = &self.end_of_sura {
                writeln!(f, "Last of sura: {}", end_sura)?;
            }

            // Add cycles information if any cycles were completed
            if self.cycle_info.cycles_completed() > 0 {
                writeln!(
                    f,
                    "Cycles completed: {}",
                    self.cycle_info.cycles_completed()
                )?;
            }

            // Add cycle distance if any cycles were tracked
            if self.cycle_info.cycle_distance() > 0.0 {
                writeln!(f, "Cycle distance: {}", self.cycle_info.cycle_distance())?;
            }

            // Add boundary crossing information if applicable
            if self.cycle_info.crossed_boundaries() {
                writeln!(f, "Crossed boundaries: Yes (wrapped around)")?;
            }

            Ok(())
        }
    }
}
