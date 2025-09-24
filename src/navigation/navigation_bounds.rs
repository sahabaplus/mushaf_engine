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
/// The bounds define a fixed range where `upper_bound` is always less than `lower_bound`:
/// - **Valid**: upper_bound: (1,1), lower_bound: (114,6)
/// - **Invalid**: upper_bound: (114,6), lower_bound: (1,1)
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
/// The bounds define a fixed range where navigation always flows from upper_bound to lower_bound:
///
/// ```rust
/// // Valid bounds: upper_bound < lower_bound
/// // upper_bound = (1,1), lower_bound = (114,6)
///
/// // Invalid bounds: upper_bound > lower_bound (will cause validation error)
/// // upper_bound = (114,6), lower_bound = (1,1) // This is invalid!
/// ```
///
/// Navigation always moves from upper_bound to lower_bound, regardless of direction.
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
    #[must_use]
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
    /// * `upper_bound` - Upper bound for navigation (must be less than lower_bound)
    /// * `lower_bound` - Lower bound for navigation (must be greater than upper_bound)
    ///
    /// # Panics
    /// Panics if `upper_bound >= lower_bound` as this violates the bounds constraint.
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
    /// ```
    #[must_use]
    pub fn new(
        iteration_limit: u32,
        upper_bound: VersePosition,
        lower_bound: VersePosition
    ) -> Self {
        assert!(upper_bound < lower_bound, "upper_bound must be less than lower_bound");
        Self { iteration_limit, upper_bound, lower_bound }
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

    /// Sets the upper bound for the bounds.
    ///
    /// # Arguments
    /// * `upper_bound` - Upper bound for navigation (must be less than lower_bound)
    ///
    /// # Panics
    /// Panics if the new upper_bound is not less than the current lower_bound.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationBounds, VersePosition};
    ///
    /// let bounds = NavigationBounds::default()
    ///     .upper_bound(VersePosition::new(2, 1)); // Upper bound at Al-Baqarah
    /// ```
    #[must_use]
    pub fn upper_bound(mut self, upper_bound: VersePosition) -> Self {
        assert!(upper_bound < self.lower_bound, "upper_bound must be less than lower_bound");
        self.upper_bound = upper_bound;
        self
    }

    /// Sets the lower bound for the bounds.
    ///
    /// # Arguments
    /// * `lower_bound` - Lower bound for navigation (must be greater than upper_bound)
    ///
    /// # Panics
    /// Panics if the new lower_bound is not greater than the current upper_bound.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_quran_engine::navigation::{NavigationBounds, VersePosition};
    ///
    /// let bounds = NavigationBounds::default()
    ///     .lower_bound(VersePosition::new(2, 286)); // Lower bound at end of Al-Baqarah
    /// ```
    #[must_use]
    pub fn lower_bound(mut self, lower_bound: VersePosition) -> Self {
        assert!(self.upper_bound < lower_bound, "lower_bound must be greater than upper_bound");
        self.lower_bound = lower_bound;
        self
    }
}
