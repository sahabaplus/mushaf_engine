mod quran_metadata;
mod verse;
mod page;
use std::{ fmt::Display, rc::Rc };
pub use page::Page;
pub use verse::Verse;
pub use quran_metadata::{ QuranMetadata, SuraInfo };

#[cfg(feature = "colored_output")]
use colored::Colorize;

#[derive(Clone, Debug)]
pub struct Mushaf {
    pub lines_per_page: u8,
    pub max_page: u16,
    pub pages: Rc<[Page]>,
}

impl Mushaf {
    pub fn new(lines_per_page: u8, max_page: u16, pages: Rc<[Page]>) -> Self {
        Self {
            lines_per_page,
            max_page,
            pages,
        }
    }

    pub fn get_page(&self, page_number: u16) -> Option<&Page> {
        if page_number == 0 || page_number > self.max_page {
            return None;
        }

        Some(&self.pages[(page_number as usize) - 1])
    }
}

impl Display for Mushaf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Display summary info with colors if enabled
        #[cfg(feature = "colored_output")]
        {
            write!(f, "{}", "=== Mushaf Summary ===\n".bright_green().bold());
            write!(f, "{}: {}\n", "Total Pages".yellow(), self.max_page.to_string().cyan());
            write!(
                f,
                "{}: {}\n",
                "Lines per Page".yellow(),
                self.lines_per_page.to_string().cyan()
            );
            write!(
                f,
                "{}: {}\n",
                "Total Verses".yellow(),
                self.pages
                    .iter()
                    .map(|p| p.verses().len())
                    .sum::<usize>()
                    .to_string()
                    .cyan()
            )
        }

        // Plain display when colored feature is not enabled
        #[cfg(not(feature = "colored_output"))]
        {
            write!(f, "=== Mushaf Summary ===\n");
            write!(f, "Total Pages: {}\n", self.max_page);
            write!(f, "Lines per Page: {}\n", self.lines_per_page);
            write!(
                f,
                "Total Verses: {}\n",
                self.pages
                    .iter()
                    .map(|p| p.verses().len())
                    .sum::<usize>()
            )
        }
    }
}
