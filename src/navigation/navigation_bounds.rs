use crate::navigation::VersePosition;

/// Defines boundaries and iteration limits for verse navigation.
///
/// `NavigationBounds` controls how the `VersesNavigator` behaves when navigating through
/// verses. It establishes a bounded range and controls how many times the navigator
/// can cycle through that range before stopping.
///
/// # Key Concepts
///
/// ## Iteration Limits
/// The `iteration_limit` controls how many complete cycles through the bounded range
/// are allowed:
/// - **0**: No iterations allowed - navigation stops at the end boundary
/// - **1**: One complete cycle through the range
/// - **n**: n complete cycles through the range
///
/// ## Bounds and Direction
/// The relationship between bounds and navigation direction is automatically handled:
/// - **Downwards navigation**: navigates from `start_position` to `end_position`
/// - **Upwards navigation**: bounds are automatically reversed internally
///
/// ## Iteration Behavior
/// When the navigator reaches the `end_position`:
/// 1. If `remaining_iterations > 0`: increment iteration count and reset to `start_position`
/// 2. If `remaining_iterations = 0`: return `None` (stop navigation)
///
/// # Examples
///
/// ```rust
/// use rust_quran_engine::navigation::{NavigationBounds, VersePosition};
///
/// // Navigate through Al-Baqarah (Sura 2) once
/// let bounds = NavigationBounds::new(
///     1,                                    // iteration_limit: 1 cycle
///     VersePosition::new(2, 1),            // start: beginning of Sura 2
///     VersePosition::new(2, 286)           // end: end of Sura 2
/// );
///
/// // Navigate through Al-Fatiha (Sura 1) twice
/// let bounds = NavigationBounds::new(
///     2,                                    // iteration_limit: 2 cycles
///     VersePosition::new(1, 1),            // start: beginning of Al-Fatiha
///     VersePosition::new(1, 7)             // end: end of Al-Fatiha
/// );
///
/// // Navigate from Al-Fatiha to Al-Baqarah without iteration
/// let bounds = NavigationBounds::new(
///     0,                                    // iteration_limit: 0 (no cycling)
///     VersePosition::new(1, 1),            // start: Al-Fatiha
///     VersePosition::new(2, 5)             // end: 5th verse of Al-Baqarah
/// );
/// ```
///
/// # When to Use Iteration Limits
///
/// ## Use iteration limits when:
/// - **Repeating recitation**: Reading the same sura multiple times
/// - **Memorization practice**: Cycling through specific verses repeatedly
/// - **Controlled navigation**: Limiting how far navigation can go
///
/// ## Don't use iteration limits when:
/// - **Linear reading**: Moving through the Quran sequentially for one go
/// - **Search operations**: Looking for specific content
/// - **One-time navigation**: Moving from point A to point B once
///
/// # Direction Interaction
///
/// The bounds are automatically adjusted based on navigation direction:
///
/// ```rust
/// // For downwards navigation (1→114):
/// // start_position = (1,1), end_position = (114,6)
///
/// // For upwards navigation (114→1):
/// // start_position = (114,6), end_position = (1,1) (automatically reversed)
/// ```
///
/// This ensures that navigation always moves from start to end, regardless of direction.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NavigationBounds {
    /// Maximum number of complete cycles through the bounded range.
    ///
    /// - **0**: No iterations - navigation stops at end boundary
    /// - **1**: One complete cycle through the range
    /// - **n**: n complete cycles through the range
    pub(super) iteration_limit: u32,

    /// Starting position for navigation within the bounded range (inclusive).
    ///
    /// When the navigator reaches the end_position and iterations remain,
    /// it resets to this position to begin a new cycle.
    pub(super) start_position: VersePosition,

    /// Ending position for navigation within the bounded range (inclusive).
    ///
    /// When this position is reached, the navigator either:
    /// - Resets to start_position (if iterations remain)
    /// - Returns None to stop navigation (if no iterations remain)
    pub(super) end_position: VersePosition,
}

impl Default for NavigationBounds {
    /// Creates default bounds with no iteration limit and full Quran range.
    ///
    /// - `iteration_limit`: 0 (no cycling)
    /// - `start_position`: (1, 1) - beginning of Al-Fatiha
    /// - `end_position`: (114, 6) - end of An-Nas
    #[must_use]
    fn default() -> Self {
        Self {
            iteration_limit: 0,
            start_position: VersePosition::start(),
            end_position: VersePosition::end(),
        }
    }
}

impl NavigationBounds {
    /// Creates a new `NavigationBounds` with specified iteration limit and positions.
    ///
    /// # Arguments
    /// * `iteration_limit` - Maximum number of complete cycles through the range
    /// * `start_position` - Starting position for navigation
    /// * `end_position` - Ending position for navigation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationBounds, VersePosition};
    ///
    /// // Create bounds for Al-Fatiha with 3 iterations
    /// let bounds = NavigationBounds::new(
    ///     3,
    ///     VersePosition::new(1, 1),
    ///     VersePosition::new(1, 7)
    /// );
    /// ```
    #[must_use]
    pub const fn new(
        iteration_limit: u32,
        start_position: VersePosition,
        end_position: VersePosition
    ) -> Self {
        Self { iteration_limit, start_position, end_position }
    }

    /// Sets the iteration limit for the bounds.
    ///
    /// # Arguments
    /// * `iteration_limit` - Maximum number of complete cycles through the range
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationBounds, VersePosition};
    ///
    /// let bounds = NavigationBounds::default()
    ///     .iteration_limit(5); // Allow 5 complete cycles
    /// ```
    #[must_use]
    pub fn iteration_limit(mut self, iteration_limit: u32) -> Self {
        self.iteration_limit = iteration_limit;
        self
    }

    /// Sets the start position for the bounds.
    ///
    /// # Arguments
    /// * `start_position` - Starting position for navigation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationBounds, VersePosition};
    ///
    /// let bounds = NavigationBounds::default()
    ///     .start_position(VersePosition::new(2, 1)); // Start from Al-Baqarah
    /// ```
    #[must_use]
    pub fn start_position(mut self, start_position: VersePosition) -> Self {
        self.start_position = start_position;
        self
    }

    /// Sets the end position for the bounds.
    ///
    /// # Arguments
    /// * `end_position` - Ending position for navigation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationBounds, VersePosition};
    ///
    /// let bounds = NavigationBounds::default()
    ///     .end_position(VersePosition::new(2, 286)); // End at Al-Baqarah
    /// ```
    #[must_use]
    pub fn end_position(mut self, end_position: VersePosition) -> Self {
        self.end_position = end_position;
        self
    }
}
