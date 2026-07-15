//! King Fahad Mushaf JSON loader (feature `king_fahad_mushaf`).
//!
//! Converts page/verse JSON into a [`crate::Mushaf`]. Prefer
//! [`crate::BaseMushafEngine::king_fahad`] when using the bundled data file.

use crate::mushaf::{Mushaf, Page, Verse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Loader for King Fahad Quran Printing Complex Mushaf JSON.
pub struct KingFahadMushaf;
impl KingFahadMushaf {
    /// Load a King Fahad Mushaf from a file
    ///
    /// # Arguments
    /// * `path` - The path to the JSON file containing the Mushaf data
    ///
    /// # Returns
    /// A `Result` containing the `Mushaf` if successful, or an `std::io::Error` if the file cannot be read or parsed
    ///
    /// # Errors
    /// * `std::io::Error` if the file cannot be read or parsed
    /// * `serde::de::Error` if the JSON is invalid
    pub fn from_file(path: &str) -> Result<Mushaf, std::io::Error> {
        // Load from provided JSON path
        let file_content = std::fs::read_to_string(path).map_err(std::io::Error::other)?;

        let pages: Vec<Vec<JsonVerse>> =
            serde_json::from_str(&file_content).map_err(std::io::Error::other)?;

        Ok(Self::create_mushaf_from_pages(pages))
    }

    /// Create a Mushaf from a vector of JSON pages
    ///
    /// # Arguments
    /// * `json_pages` - A vector of JSON pages
    ///
    /// # Returns
    /// A `Result` containing the `Mushaf` if successful, or an `std::io::Error` if the pages cannot be created
    ///
    /// # Errors
    /// * `std::io::Error` if the pages cannot be created
    /// * `serde::de::Error` if the JSON is invalid
    ///
    /// # Panics
    /// * `expect` if the page number cannot be converted to a `u16`
    #[must_use]
    pub fn create_mushaf_from_pages(json_pages: Vec<Vec<JsonVerse>>) -> Mushaf {
        let mut pages = Vec::with_capacity(json_pages.len());

        for (i, page_verses) in json_pages.into_iter().enumerate() {
            let i = u16::try_from(i).expect("expect `i` to be less than `u16::MAX`");
            let page_number: u16 = i + 1;

            // Create a Vec<Verse> first
            let verses: Vec<Verse> = page_verses
                .iter()
                .map(|v| Verse {
                    sura: v.sura,
                    number: v.ayah,
                    position: (v.x, v.y),
                    lines: v.lines,
                })
                .collect();

            // Then convert to Arc<[Verse]>
            let verses_rc = Arc::from(verses);

            pages.push(Page::new(page_number, verses_rc));
        }

        Mushaf {
            lines_per_page: 15,
            max_page: u16::try_from(pages.len())
                .expect("expect `pages.len()` to be less than `u16::MAX`"),
            pages: Arc::from(pages),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct JsonVerse {
    sura: u8,
    ayah: u16,
    lines: f32,
    y: u8,
    x: f32,
}

#[cfg(test)]
mod test {
    use crate::king_fahad_mushaf::KingFahadMushaf;
    use colored::Colorize;

    use std::path::PathBuf;

    #[test]
    fn load_mushaf() {
        // Use a relative path to the data directory in your project
        let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data_path.push("data");
        data_path.push("king_fahad_mushaf.json");

        let mushaf = KingFahadMushaf::from_file(
            data_path.to_str().expect("expect `data_path` to be a valid string"),
        );
        let mushaf = mushaf.expect("expect `mushaf` not to be `Err`");

        // Add some assertions to actually test something
        assert!(mushaf.max_page > 0);
        assert_eq!(mushaf.lines_per_page, 15);
        assert!(!mushaf.pages.is_empty());

        let pages = mushaf.pages;
        for page in pages.iter() {
            // let page = &pages[i];
            let suras_headers: f32 = page
                .verses()
                .iter()
                .map(|v| {
                    if v.number == 1 {
                        if v.sura == 9 {
                            // سورة التوبة, does not have بسملة
                            1f32
                        } else {
                            2f32
                        }
                    } else {
                        0f32
                    }
                })
                .sum();
            let sum: f32 = page
                .verses()
                .iter()
                .fold(0f32, |acc, e| ((acc + e.lines) * 100.0).round() / 100.0)
                + suras_headers;
            println!("{:#?}", page);
            println!(
                "\t{}:{}\n",
                "Sum".to_string().bold().green(),
                sum.to_string().yellow().bold()
            );
            if page.number() > 2 {
                assert_eq!(sum, 15f32);
            }
        }
    }
}
