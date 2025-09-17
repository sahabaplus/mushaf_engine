use crate::navigation::{ NavigationBounds, VersePosition };

/// Comprehensive settings for verse navigation behavior.
///
/// `NavigationSettings` combines navigation bounds with additional behavioral options
/// to control how the `VersesNavigator` operates. It serves as the central configuration
/// for all navigation operations.
///
/// # Key Features
///
/// ## Bounds Control
/// The `bounds` field contains a `NavigationBounds` that defines:
/// - Iteration limits (how many cycles through a range)
/// - Start and end positions for navigation
///
/// ## Header Handling
/// The `ignore_sura_header` field controls whether sura headers (bismillah and sura titles)
/// are included in line calculations:
/// - `false` (default): Include sura headers in line calculations
/// - `true`: Exclude sura headers from line calculations
///
/// # Direction-Aware Behavior
///
/// Navigation settings automatically adapt to direction changes:
/// - When direction changes, bounds are automatically reversed via `reverse_bounds()`
/// - This ensures navigation always flows from start to end position
///
/// # Examples
///
/// ## Basic Usage
/// ```rust
/// use rust_quran_engine::navigation::{NavigationSettings, VersePosition};
///
/// // Create settings for reading Al-Fatiha twice with headers
/// let settings = NavigationSettings::builder()
///     .start_position(VersePosition::new(1, 1))
///     .end_position(VersePosition::new(1, 7))
///     .iteration_limit(2)
///     .ignore_sura_header(false); // Include headers
/// ```
///
/// ## Page Layout Calculation
/// ```rust
/// use rust_quran_engine::navigation::{NavigationSettings, VersePosition};
///
/// // Calculate lines without sura headers for precise page layout
/// let settings = NavigationSettings::builder()
///     .start_position(VersePosition::new(2, 1))
///     .end_position(VersePosition::new(2, 50))
///     .iteration_limit(0) // No cycling needed for layout
///     .ignore_sura_header(true); // Exclude headers for layout
/// ```
///
/// ## Memorization Practice
/// ```rust
/// use rust_quran_engine::navigation::{NavigationSettings, VersePosition};
///
/// // Practice specific verses multiple times
/// let settings = NavigationSettings::builder()
///     .start_position(VersePosition::new(67, 1))
///     .end_position(VersePosition::new(67, 30))
///     .iteration_limit(5) // Practice 5 times
///     .ignore_sura_header(false); // Include full context
/// ```
///
/// # When to Use Each Setting
///
/// ## `ignore_sura_header: false` (default)
/// Use when:
/// - **Page layout calculations**: Page lines end up having 15 lines wither containing headers of suras or normal full page of verses
///
/// ## `ignore_sura_header: true`
/// Use when:
/// - **Accuracy**: Calculating exact verse lines without headers
///
/// ## Iteration Limits
/// See `NavigationBounds` documentation for detailed iteration behavior.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NavigationSettings {
    /// Whether to include sura headers (bismillah and sura titles) in line calculations.
    ///
    /// - `false` (default): Include sura headers in calculations
    /// - `true`: Exclude sura headers from calculations
    ///
    /// This affects the `calculate_verse_lines()` method and is useful for:
    /// - Page layout calculations (exclude headers)
    /// - Display purposes (include headers)
    pub ignore_sura_header: bool,

    /// Navigation bounds controlling iteration limits and position boundaries.
    ///
    /// Contains the iteration limit and start/end positions for navigation.
    /// See `NavigationBounds` documentation for detailed behavior.
    pub bounds: NavigationBounds,
}

impl NavigationSettings {
    /// Creates new navigation settings with specified header handling and bounds.
    ///
    /// # Arguments
    /// * `ignore_sura_header` - Whether to exclude sura headers from line calculations
    /// * `bounds` - Navigation bounds containing iteration limits and positions
    #[must_use]
    pub const fn new(ignore_sura_header: bool, bounds: NavigationBounds) -> Self {
        Self { ignore_sura_header, bounds }
    }

    /// Creates a builder for navigation settings with default values.
    ///
    /// Default values:
    /// - `ignore_sura_header`: false (include headers)
    /// - `iteration_limit`: 0 (no cycling)
    /// - `start_position`: (1, 1) - beginning of Al-Fatiha
    /// - `end_position`: (114, 6) - end of An-Nas
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationSettings, VersePosition};
    ///
    /// let settings = NavigationSettings::builder()
    ///     .start_position(VersePosition::new(2, 1))
    ///     .end_position(VersePosition::new(2, 50))
    ///     .iteration_limit(1)
    ///     .ignore_sura_header(true);
    /// ```
    #[must_use]
    pub fn builder() -> Self {
        Self::new(Default::default(), Default::default())
    }

    /// Sets whether to ignore sura headers in line calculations.
    ///
    /// # Arguments
    /// * `ignore_sura_header` - Whether to exclude sura headers from calculations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::NavigationSettings;
    ///
    /// let settings = NavigationSettings::builder()
    ///     .ignore_sura_header(true); // Exclude headers for layout calculations
    /// ```
    #[must_use]
    pub fn ignore_sura_header(mut self, ignore_sura_header: bool) -> Self {
        self.ignore_sura_header = ignore_sura_header;
        self
    }

    /// Sets the navigation bounds directly.
    ///
    /// # Arguments
    /// * `bounds` - Complete navigation bounds configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationSettings, NavigationBounds, VersePosition};
    ///
    /// let bounds = NavigationBounds::new(
    ///     3,
    ///     VersePosition::new(1, 1),
    ///     VersePosition::new(1, 7)
    /// );
    /// let settings = NavigationSettings::builder()
    ///     .bounds(bounds);
    /// ```
    #[must_use]
    pub fn bounds(mut self, bounds: NavigationBounds) -> Self {
        self.bounds = bounds;
        self
    }

    /// Sets the iteration limit for the navigation bounds.
    ///
    /// # Arguments
    /// * `iteration_limit` - Maximum number of complete cycles through the range
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationSettings, VersePosition};
    ///
    /// let settings = NavigationSettings::builder()
    ///     .iteration_limit(5); // Allow 5 complete cycles
    /// ```
    #[must_use]
    pub fn iteration_limit(mut self, iteration_limit: u32) -> Self {
        self.bounds.iteration_limit = iteration_limit;
        self
    }

    /// Sets the start position for navigation.
    ///
    /// # Arguments
    /// * `start_position` - Starting position for navigation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationSettings, VersePosition};
    ///
    /// let settings = NavigationSettings::builder()
    ///     .start_position(VersePosition::new(2, 1)); // Start from Al-Baqarah
    /// ```
    #[must_use]
    pub fn start_position(mut self, start_position: VersePosition) -> Self {
        self.bounds.start_position = start_position;
        self
    }

    /// Sets the end position for navigation.
    ///
    /// # Arguments
    /// * `end_position` - Ending position for navigation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationSettings, VersePosition};
    ///
    /// let settings = NavigationSettings::builder()
    ///     .end_position(VersePosition::new(2, 286)); // End at Al-Baqarah
    /// ```
    #[must_use]
    pub fn end_position(mut self, end_position: VersePosition) -> Self {
        self.bounds.end_position = end_position;
        self
    }

    /// Reverses the start and end positions of the navigation bounds.
    ///
    /// This method is automatically called when the navigation direction changes
    /// to ensure navigation always flows from start to end position regardless
    /// of the chosen direction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationSettings, VersePosition};
    ///
    /// let settings = NavigationSettings::builder()
    ///     .start_position(VersePosition::new(1, 1))
    ///     .end_position(VersePosition::new(1, 7))
    ///     .reverse_bounds(); // Now start=(1,7), end=(1,1)
    /// ```
    ///
    /// # Direction Interaction
    ///
    /// This method is typically used internally when:
    /// - Changing from `Direction::Downwards` to `Direction::Upwards`
    /// - Changing from `Direction::Upwards` to `Direction::Downwards`
    ///
    /// The navigator automatically calls this to maintain consistent behavior.
    #[must_use]
    pub fn reverse_bounds(mut self) -> Self {
        ::std::mem::swap(&mut self.bounds.start_position, &mut self.bounds.end_position);
        self
    }
}

impl Default for NavigationSettings {
    /// Creates default navigation settings.
    ///
    /// Default values:
    /// - `ignore_sura_header`: false (include sura headers)
    /// - `iteration_limit`: 0 (no cycling)
    /// - `start_position`: (1, 1) - beginning of Al-Fatiha
    /// - `end_position`: (114, 6) - end of An-Nas
    #[must_use]
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
    }
}
