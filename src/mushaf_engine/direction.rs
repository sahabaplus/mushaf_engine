/// Direction for navigating through the Quran
///
/// # Variants
///
/// * `Downwards` - Navigate from Sura Al-Fatiha (1) towards Sura An-Nas (114)
/// * `Upwards` - Navigate from Sura An-Nas (114) towards Sura Al-Fatiha (1)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    #[default]
    Downwards,
    Upwards,
}
