/// Settings for navigation
///
/// This struct contains settings for navigation, including whether to ignore sura headers and the number of iterations.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NavigationSettings {
    /// Whether to ignore sura headers.
    pub ignore_sura_header: bool,
}

impl NavigationSettings {
    #[must_use]
    pub const fn new(ignore_sura_header: bool) -> Self {
        Self { ignore_sura_header }
    }
}

impl Default for NavigationSettings {
    #[must_use]
    fn default() -> Self {
        Self::new(false)
    }
}
