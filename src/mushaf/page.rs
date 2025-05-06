use std::{ fmt::Debug, rc::Rc };

use super::verse::Verse;

#[cfg(feature = "colored_output")]
use colored::Colorize;

#[derive(Default, Clone)]
pub struct Page {
    number: u16,
    verses: Rc<[Verse]>,
}

impl Page {
    pub fn new(number: u16, verses: Rc<[Verse]>) -> Page {
        Page {
            number,
            verses,
        }
    }

    pub fn number(&self) -> u16 {
        self.number
    }

    pub fn verses(&self) -> &[Verse] {
        &self.verses
    }

    // Method to print page info with colored output if enabled
}

#[cfg(not(feature = "colored_output"))]
impl std::fmt::Debug for Verse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Verse")
            .field("sura", &self.sura)
            .field("number", &self.number)
            .field("position", &self.position)
            .field("lines", &self.lines)
            .finish()
    }
}

#[cfg(feature = "colored_output")]
impl Debug for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // pub fn display(&self) {
        write!(f, "{} {}\n", "Page:".bright_green().bold(), self.number.to_string().yellow());

        write!(f, "\t{}\n", "Verses:".bright_green().bold());
        for (i, verse) in self.verses.iter().enumerate() {
            write!(f, "\t\t{:2}- {}\n", (i + 1).to_string().blue(), verse);
        }

        Ok(())
    }
}
