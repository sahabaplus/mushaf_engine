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
/// The bounds can work in two modes:
/// - **Inclusive mode** (`upper_bound < lower_bound`): Navigate from upper_bound to lower_bound (inclusive)
///   - Example: upper_bound: (1,1), lower_bound: (114,6) - navigate entire Quran
/// - **Excluding mode** (`upper_bound > lower_bound`): Navigate on the whole range EXCLUDING the range from lower_bound to upper_bound (inclusive)
///   - Example: upper_bound: (2,50), lower_bound: (2,1) - navigate everywhere except verses 1-50 of Sura 2
///
/// ## Iteration Behavior
/// When the navigator reaches the `lower_bound`:
/// 1. If `remaining_iterations > 0`: increment iteration count and reset to `upper_bound`
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
///     VersePosition::new(2, 1),            // upper_bound: beginning of Sura 2
///     VersePosition::new(2, 286)           // lower_bound: end of Sura 2
/// );
///
/// // Navigate through Al-Fatiha (Sura 1) twice
/// let bounds = NavigationBounds::new(
///     2,                                    // iteration_limit: 2 cycles
///     VersePosition::new(1, 1),            // upper_bound: beginning of Al-Fatiha
///     VersePosition::new(1, 7)             // lower_bound: end of Al-Fatiha
/// );
///
/// // Navigate from Al-Fatiha to Al-Baqarah without iteration
/// let bounds = NavigationBounds::new(
///     0,                                    // iteration_limit: 0 (no cycling)
///     VersePosition::new(1, 1),            // upper_bound: Al-Fatiha
///     VersePosition::new(2, 5)             // lower_bound: 5th verse of Al-Baqarah
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
/// The bounds can work in two modes:
///
/// ```rust
/// use rust_quran_engine::navigation::{NavigationBounds, VersePosition};
///
/// // Inclusive mode: upper_bound < lower_bound
/// // Navigate from (1,1) to (114,6) - entire Quran
/// let bounds = NavigationBounds::new(0, VersePosition::new(1, 1), VersePosition::new(114, 6));
///
/// // Excluding mode: upper_bound > lower_bound
/// // Navigate everywhere EXCEPT from (2,1) to (2,50)
/// let bounds = NavigationBounds::new(0, VersePosition::new(2, 50), VersePosition::new(2, 1));
/// ```
///
/// In inclusive mode, navigation moves from upper_bound to lower_bound.
/// In excluding mode, navigation skips the excluded range automatically.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NavigationBounds {
    /// Maximum number of complete cycles through the bounded range.
    ///
    /// - **0**: No iterations - navigation stops at end boundary
    /// - **1**: One complete cycle through the range
    /// - **n**: n complete cycles through the range
    pub(super) iteration_limit: u32,

    /// Upper bound for navigation within the bounded range (inclusive).
    ///
    /// This must be less than lower_bound. When the navigator reaches the lower_bound
    /// and iterations remain, it resets to this position to begin a new cycle.
    pub(super) upper_bound: VersePosition,

    /// Lower bound for navigation within the bounded range (inclusive).
    ///
    /// This must be greater than upper_bound. When this position is reached, the navigator either:
    /// - Resets to upper_bound (if iterations remain)
    /// - Returns None to stop navigation (if no iterations remain)
    pub(super) lower_bound: VersePosition,
}

impl Default for NavigationBounds {
    /// Creates default bounds with no iteration limit and full Quran range.
    ///
    /// - `iteration_limit`: 0 (no cycling)
    /// - `upper_bound`: (1, 1) - beginning of Al-Fatiha
    /// - `lower_bound`: (114, 6) - end of An-Nas
    fn default() -> Self {
        Self {
            iteration_limit: 0,
            upper_bound: VersePosition::start(),
            lower_bound: VersePosition::end(),
        }
    }
}

impl NavigationBounds {
    /// Creates a new `NavigationBounds` with specified iteration limit and positions.
    ///
    /// # Arguments
    /// * `iteration_limit` - Maximum number of complete cycles through the range
    /// * `upper_bound` - Upper bound for navigation
    /// * `lower_bound` - Lower bound for navigation
    ///
    /// # Bounds Behavior
    /// - When `upper_bound < lower_bound`: Navigate from upper_bound to lower_bound (inclusive)
    /// - When `upper_bound > lower_bound`: Navigate on the whole range EXCLUDING the range from lower_bound to upper_bound (inclusive)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationBounds, VersePosition};
    ///
    /// // Create bounds for Al-Fatiha with 3 iterations
    /// let bounds = NavigationBounds::new(
    ///     3,
    ///     VersePosition::new(1, 1),  // upper_bound
    ///     VersePosition::new(1, 7)   // lower_bound
    /// );
    ///
    /// // Exclude a range: navigate everywhere except from (2, 1) to (2, 50)
    /// let bounds = NavigationBounds::new(
    ///     0,
    ///     VersePosition::new(2, 50),  // upper_bound > lower_bound
    ///     VersePosition::new(2, 1)      // lower_bound
    /// );
    /// ```
    #[must_use]
    pub fn new(
        iteration_limit: u32,
        upper_bound: VersePosition,
        lower_bound: VersePosition,
    ) -> Self {
        Self {
            iteration_limit,
            upper_bound,
            lower_bound,
        }
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

    /// Checks if the bounds are in excluding mode.
    ///
    /// # Returns
    /// * `true` - Excluding mode (upper_bound > lower_bound): navigate everywhere except the excluded range
    /// * `false` - Inclusive mode (upper_bound <= lower_bound): navigate within the range
    ///
    /// # Note
    /// Excluding mode currently only works correctly with `Direction::Downwards`.
    /// `Direction::Upwards` uses per-sura forward navigation and doesn't support
    /// verse-by-verse backwards navigation through excluded ranges.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationBounds, VersePosition};
    ///
    /// // Inclusive mode
    /// let inclusive = NavigationBounds::new(0, VersePosition::new(1, 1), VersePosition::new(1, 7));
    /// assert!(!inclusive.is_excluding_mode());
    ///
    /// // Excluding mode
    /// let excluding = NavigationBounds::new(0, VersePosition::new(1, 7), VersePosition::new(1, 1));
    /// assert!(excluding.is_excluding_mode());
    /// ```
    #[must_use]
    pub fn is_excluding_mode(&self) -> bool {
        self.upper_bound > self.lower_bound
    }

    /// Get the upper bound
    #[must_use]
    pub fn upper_bound(&self) -> VersePosition {
        self.upper_bound
    }

    /// Get the lower bound
    #[must_use]
    pub fn lower_bound(&self) -> VersePosition {
        self.lower_bound
    }

    /// Get the iteration limit (builder method is also named `iteration_limit`).
    #[must_use]
    pub const fn get_iteration_limit(&self) -> u32 {
        self.iteration_limit
    }
}
