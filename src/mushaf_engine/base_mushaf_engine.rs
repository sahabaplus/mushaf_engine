use crate::{
    mushaf::{ Mushaf, QuranMetadata, SuraInfo, Verse },
    navigation::{ Direction, LastVerseResult, NavigationResult, OverflowResult, VersesNavigator },
};
use std::rc::Rc;

use super::{ IMushafEngine };

/// Basic implementation of the Mushaf navigation engine using King Fahad Mushaf
pub struct BaseMushafEngine {
    pub mushaf: Rc<Mushaf>,
    pub quran_metadata: Rc<QuranMetadata>,
}

impl BaseMushafEngine {
    /// Create a new BaseMushafEngine with the given Mushaf
    pub fn new(mushaf: Rc<Mushaf>) -> Self {
        let quran_metadata = Rc::new(QuranMetadata::from_mushaf(&mushaf));
        BaseMushafEngine { mushaf, quran_metadata }
    }

    /// Get metadata about a specific Sura
    pub fn get_sura_info(&self, sura_number: u8) -> Option<&SuraInfo> {
        self.quran_metadata.get_sura_info(sura_number)
    }

    /// Find a verse in the mushaf and return its location (Verse, page, index_of_verse)
    fn find_verse(&self, sura_number: u8, verse_number: u16) -> Option<(&Verse, u16, u8)> {
        let sura = self.quran_metadata.get_sura_info(sura_number);
        if let Some(sura) = sura {
            let pages = self.mushaf.pages.as_ref();
            for i in sura.start_page..=sura.end_page {
                let page = &pages[(i - 1) as usize];
                for (i, verse) in page.verses().iter().enumerate() {
                    if verse.sura == sura_number && verse.number == verse_number {
                        return Some((verse, page.number(), i as u8));
                    }
                }
            }
            None
        } else {
            None
        }
    }

    fn calculate_lines(
        &self,
        start_sura: u8,
        start_verse: u16,
        end_sura: u8,
        end_verse: u16,
        direction: Direction
    ) -> f32 {
        assert!(
            (start_sura < end_sura && direction == Direction::Downwards) ||
                (start_sura > end_sura && direction == Direction::Upwards) ||
                start_sura == end_sura
        );

        let start_verse = self.find_verse(start_sura, start_verse).expect("Verse not found").0;
        let end_verse = self.find_verse(start_sura, end_verse).expect("Verse not found").0;
        let mut navigator = VersesNavigator::new(self.mushaf.clone(), self.quran_metadata.clone());
        navigator.reset_position(start_verse.sura, start_verse.number);

        let mut lines = 0f32;
        loop {
            // lines += navigator.current_verse().lines;
            lines = ((lines + navigator.current_verse().lines) * 100.0).round() / 100.0;
            if *navigator.current_verse() == *end_verse {
                break;
            }
            navigator.next_verse(direction);
        }

        lines
    }

    /// Calculate the lines taken by a verse including any sura headers
    fn calculate_verse_lines(&self, verse: &Verse) -> f32 {
        let mut total_lines = verse.lines;

        // Add lines for sura headers if this is the first verse of a sura
        if verse.number == 1 {
            // All suras except At-Tawbah (9) have bismillah
            if verse.sura != 9 {
                total_lines += 2.0; // Sura title + bismillah
            } else {
                total_lines += 1.0; // Only sura title for At-Tawbah
            }
        }

        total_lines
    }

    fn is_in_opposite_direction(start_sura: u8, end_sura: u8, direction: Direction) -> bool {
        !(
            (start_sura < end_sura && direction == Direction::Downwards) ||
            (start_sura > end_sura && direction == Direction::Upwards) ||
            start_sura == end_sura
        )
    }

    fn prefer_last_of_sura(
        &self,
        last_of_sura: Option<LastVerseResult>,
        current_verse: &Verse,
        direction: Direction
    ) -> LastVerseResult {
        if
            last_of_sura.is_some() &&
            current_verse.sura == last_of_sura.as_ref().unwrap().last_verse.sura
        {
            return last_of_sura.unwrap();
        }

        let sura = self.quran_metadata.get_sura_info(current_verse.sura).unwrap();
        let last_verse = self.find_verse(sura.number, sura.total_verses).unwrap().0;
        let lines_distance =
            self.calculate_lines(
                current_verse.sura,
                current_verse.number,
                last_verse.sura,
                last_verse.number,
                direction
            ) - current_verse.lines;
        if
            last_of_sura.is_some() &&
            last_of_sura.as_ref().unwrap().lines_distance.abs() + 3.0 < lines_distance.abs() // added 3 to prefer the new last
        {
            last_of_sura.unwrap()
        } else {
            LastVerseResult::new(lines_distance, *last_verse)
        }
    }
    fn prefer_last_of_page(
        &self,
        last_of_page: Option<LastVerseResult>,
        current_verse: &Verse,
        direction: Direction
    ) -> Option<LastVerseResult> {
        if last_of_page.is_some() && *current_verse == last_of_page.as_ref().unwrap().last_verse {
            return last_of_page;
        }

        let (_, page, idx) = self.find_verse(current_verse.sura, current_verse.number).unwrap();
        let new_last_of_page = self.mushaf.get_page(page).unwrap().verses().last().unwrap();
        if Self::is_in_opposite_direction(current_verse.sura, new_last_of_page.sura, direction) {
            return last_of_page;
        }

        let lines_distance = self.calculate_lines(
            current_verse.sura,
            current_verse.number,
            new_last_of_page.sura,
            new_last_of_page.number,
            direction
        );
        if
            last_of_page.is_some() &&
            last_of_page.as_ref().unwrap().lines_distance.abs() + 1.0 < lines_distance.abs() // added 1 to prefer the new last
        {
            last_of_page
        } else {
            Some(LastVerseResult::new(lines_distance, *new_last_of_page))
        }
    }
}

impl IMushafEngine for BaseMushafEngine {
    fn navigate(
        &self,
        lines: f32,
        from_sura: u8,
        from_verse: u16,
        direction: Direction
    ) -> NavigationResult {
        assert!(lines >= 0f32, "lines should be positive or zero");
        // If no lines to navigate, find and return the current verse
        if lines == 0f32 {
            if let Some((verse, _, _)) = self.find_verse(from_sura, from_verse) {
                return NavigationResult::new_normal(*verse, 0f32);
            }
            panic!("Verse Not Found!");
        }

        // Find starting position
        let verse = match self.find_verse(from_sura, from_verse) {
            Some(verse) => verse,
            _ => panic!("Verse Not Found!"),
        };

        let mut remaining_lines = lines;

        let (verse, page, verse_idx) = verse;

        let mut navigator = VersesNavigator::new(self.mushaf.clone(), self.quran_metadata.clone());
        navigator.reset_position(verse.sura, verse.number);

        let mut overflow: Option<OverflowResult> = None;
        let mut previous_verse = *navigator.current_verse();

        let mut last_of_page: Option<LastVerseResult> = None;
        let mut last_of_sura: Option<LastVerseResult> = None;
        loop {
            let current_verse = navigator.current_verse();
            let current_sura_info = self.quran_metadata.get_sura_info(current_verse.sura).unwrap(); // can be optimized
            let verse_lines = self.calculate_verse_lines(current_verse);
            let diff = ((remaining_lines - verse_lines) * 100.0).round() / 100.0;

            if current_verse.is_last_of_page() {
                last_of_page = Some(LastVerseResult::new(-1.0 * diff, *current_verse));
            }

            if current_sura_info.total_verses == current_verse.number {
                last_of_sura = Some(LastVerseResult::new(-1.0 * diff, *current_verse));
            }

            if diff < 0.0 {
                overflow = Some(OverflowResult::new(-1.0 * diff, *current_verse));
                break;
            }

            remaining_lines = diff;
            previous_verse = *current_verse;
            if remaining_lines > 0.0 {
                navigator.next_verse(direction);
            } else {
                break;
            }
        }

        let last_of_sura = self.prefer_last_of_sura(last_of_sura, &previous_verse, direction);
        let last_of_page = self.prefer_last_of_page(last_of_page, &previous_verse, direction);

        NavigationResult::new(
            previous_verse,
            overflow,
            last_of_page,
            Some(last_of_sura),
            lines - remaining_lines
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{ path::PathBuf, rc::Rc };
    use colored::Colorize;
    use crate::{
        king_fahad_mushaf::KingFahadMushaf,
        mushaf_engine::{ base_mushaf_engine::BaseMushafEngine, IMushafEngine },
        navigation::{ Direction, NavigationResult },
    };

    fn setup_engine() -> BaseMushafEngine {
        let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data_path.push("data");
        data_path.push("king_fahad_mushaf.json");

        let mushaf = Rc::new(KingFahadMushaf::from_file(data_path.to_str().unwrap()));
        BaseMushafEngine::new(mushaf)
    }

    #[test]
    fn navigate_15_lines() {
        let engine = setup_engine();
        let lines = 15.1f32;
        let result = engine.navigate(lines, 5, 3, Direction::Upwards);
        println!("{:16}{}", "Lines: ".bold().cyan(), lines.to_string().yellow().bold());
        println!("{}", result);
        let NavigationResult {
            distance_moved,
            end_of_page,
            end_of_sura,
            overflow,
            remaining_distance,
            verse,
        } = result;

        assert_eq!(distance_moved, 15.0);
        assert_eq!(verse.sura, 5);
        assert_eq!(verse.number, 5);
        assert!(end_of_page.is_some());
        assert!(end_of_sura.is_some());
        assert!(overflow.is_some());
        assert_eq!(overflow.unwrap().overflow_lines, 7.9);
        assert_eq!(end_of_page.unwrap().lines_distance, -0.1);
        assert_eq!(end_of_sura.unwrap().lines_distance, 300.0);
    }
    #[test]
    fn navigation_overflow() {
        let engine = setup_engine();

        let v = engine.navigate(14.7f32, 114, 1, Direction::Upwards);

        println!("{}", v);
        assert!(v.overflow.is_some());
        assert!(v.end_of_sura.is_some());
        assert_eq!(v.overflow.unwrap().overflowed_verse, v.end_of_sura.unwrap().last_verse);
        assert_eq!(v.verse.sura, 112);
        assert_eq!(v.verse.number, 3);
    }

    #[test]
    fn navigate_zero_lines() {
        let engine = setup_engine();

        // First verse of Al-Fatiha
        let result = engine.navigate(0f32, 1, 1, Direction::Downwards);
        assert_eq!(result.verse.sura, 1);
        assert_eq!(result.verse.number, 1);
    }
}
