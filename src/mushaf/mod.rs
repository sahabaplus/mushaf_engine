mod quran_metadata;
mod verse;
mod page;
use std::rc::Rc;
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

    // Display summary info with colors if enabled
    #[cfg(feature = "colored_output")]
    pub fn display_summary(&self) {
        println!("{}", "=== Mushaf Summary ===".bright_green().bold());
        println!("{}: {}", "Total Pages".yellow(), self.max_page.to_string().cyan());
        println!("{}: {}", "Lines per Page".yellow(), self.lines_per_page.to_string().cyan());
        println!(
            "{}: {}",
            "Total Verses".yellow(),
            self.pages
                .iter()
                .map(|p| p.verses().len())
                .sum::<usize>()
                .to_string()
                .cyan()
        );
    }

    // Plain display when colored feature is not enabled
    #[cfg(not(feature = "colored_output"))]
    pub fn display_summary(&self) {
        println!("=== Mushaf Summary ===");
        println!("Total Pages: {}", self.max_page);
        println!("Lines per Page: {}", self.lines_per_page);
        println!(
            "Total Verses: {}",
            self.pages
                .iter()
                .map(|p| p.verses().len())
                .sum::<usize>()
        );
    }
}
