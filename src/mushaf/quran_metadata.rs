use super::Mushaf;
use crate::navigation::LookupError;

/// Information about a Sura in the Quran
#[derive(Debug, Clone)]
pub struct SuraInfo {
    /// Number of the Sura (1-114)
    pub number: u8,
    /// Total number of verses in the Sura
    pub total_verses: u16,
    /// Total lines without header (bismillah and title)
    pub lines: f32,
    /// Total lines including header
    pub lines_with_header: f32,
    /// Page number where the Sura starts
    pub start_page: u16,
    /// Page number where the Sura ends
    pub end_page: u16,
}

/// Collection of Sura information for quick lookup
#[derive(Debug, Clone)]
pub struct QuranMetadata {
    /// Array of Sura information indexed by Sura number (0-indexed for internal storage)
    /// Access using `get_sura_info()` method to convert 1-based sura numbers
    suras: Vec<SuraInfo>,
}

impl QuranMetadata {
    /// Create a new `QuranMetadata` from the Mushaf
    ///
    /// # Panics
    /// * `expect` if the sura number cannot be converted to a `u8`
    #[must_use]
    pub fn from_mushaf(mushaf: &Mushaf) -> Self {
        // Create a Vec with capacity for exactly 114 suras (0-indexed internally)
        let mut suras = Vec::with_capacity(114);

        // Maps to track sura statistics (1-indexed for easier reading)
        let mut current_sura = 0;
        let mut sura_verse_count = [0u16; 115]; // Use 115 for 1-based indexing
        let mut sura_lines = [0.0f32; 115];
        let mut sura_start_page = [0u16; 115];
        let mut sura_end_page = [0u16; 115];

        // Scan through the mushaf to collect information
        for page in mushaf.pages.iter() {
            for verse in page.verses() {
                let sura_num = verse.sura as usize;

                // Track when we enter a new sura
                if sura_num != current_sura {
                    current_sura = sura_num;
                    sura_start_page[sura_num] = page.number();
                }

                // Count verses and lines
                sura_verse_count[sura_num] += 1;
                sura_lines[sura_num] += verse.lines;

                // Update end page
                sura_end_page[sura_num] = page.number();
            }
        }

        // Create SuraInfo for each sura (store in 0-indexed array)
        for sura_num in 1..=114 {
            // Add header lines for each sura
            let header_lines = if sura_num == 9 { 1.0 } else { 2.0 };
            let total_lines_with_header = sura_lines[sura_num] + header_lines;

            suras.push(SuraInfo {
                number: u8::try_from(sura_num)
                    .expect("expect `sura_num` to be less than `u8::MAX`"),
                total_verses: sura_verse_count[sura_num],
                lines: sura_lines[sura_num],
                lines_with_header: total_lines_with_header,
                start_page: sura_start_page[sura_num],
                end_page: sura_end_page[sura_num],
            });
        }

        Self { suras }
    }

    /// Convert 1-based sura number to 0-based index for internal storage
    #[inline]
    const fn sura_to_index(sura_number: u8) -> Option<usize> {
        if Self::is_valid_sura(sura_number) {
            Some((sura_number - 1) as usize)
        } else {
            None
        }
    }

    const fn is_valid_sura(sura_number: u8) -> bool {
        sura_number >= 1 && sura_number <= 114
    }

    /// Get information about a specific Sura (using 1-based sura number)
    ///
    /// # Errors
    /// * `LookupError::InvalidSura` if the sura number is invalid
    /// * `LookupError::SuraNotFound` if the sura number is not found
    pub fn get_sura_info(&self, sura_number: u8) -> Result<&SuraInfo, LookupError> {
        if !Self::is_valid_sura(sura_number) {
            return Err(LookupError::InvalidSura(sura_number));
        }
        Self::sura_to_index(sura_number)
            .map(|idx| &self.suras[idx])
            .ok_or(LookupError::SuraNotFound(sura_number))
    }

    /// Find which Sura contains a particular page
    #[must_use]
    pub fn find_sura_by_page(&self, page_number: u16) -> Option<Vec<u8>> {
        let vec: Vec<u8> = self
            .suras
            .iter()
            .filter(|info| info.start_page <= page_number && info.end_page >= page_number)
            .map(|info| info.number)
            .collect();

        if vec.is_empty() { None } else { Some(vec) }
    }

    /// Get total number of lines for a range of Suras
    ///
    /// # Errors
    /// * `LookupError::InvalidSura` if the sura number is invalid
    /// * `LookupError::SuraNotFound` if the sura number is not found
    ///
    /// # Panics
    /// * `expect` if `get_sura_info` fails
    pub fn get_lines_range(&self, start_sura: u8, end_sura: u8) -> Result<f32, LookupError> {
        if !Self::is_valid_sura(start_sura) {
            return Err(LookupError::InvalidSura(start_sura));
        }
        if !Self::is_valid_sura(end_sura) {
            return Err(LookupError::InvalidSura(end_sura));
        }

        let sum = (start_sura..=end_sura)
            .map(|sura_num| {
                self.get_sura_info(sura_num).expect("expect `get_sura_info` to succeed")
            })
            .map(|info| info.lines_with_header)
            .sum();
        Ok(sum)
    }

    /// Get the number of Suras in the Quran
    #[inline]
    #[must_use]
    pub fn total_suras(&self) -> usize {
        self.suras.len()
    }

    /// Get an iterator over all Sura information
    pub fn iter(&self) -> impl Iterator<Item = &SuraInfo> {
        self.suras.iter()
    }
}

#[cfg(test)]
#[path = "quran_metadata_tests.rs"]
mod tests;
