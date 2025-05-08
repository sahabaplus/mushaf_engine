use serde::{ de::Visitor, Deserialize, Serialize };
use std::rc::Rc;
use crate::mushaf::{ Mushaf, Page, Verse };

use super::*;

pub struct KingFahadMushaf;
impl KingFahadMushaf {
    pub fn new(path: &str) -> Mushaf {
        // Load from provided JSON path
        let file_content = std::fs::read_to_string(path).expect("Failed to read mushaf data file");

        let pages: Vec<Vec<JsonVerse>> = serde_json
            ::from_str(&file_content)
            .expect("Failed to parse mushaf JSON data");

        Self::create_mushaf_from_pages(pages)
    }

    fn create_mushaf_from_pages(json_pages: Vec<Vec<JsonVerse>>) -> Mushaf {
        let mut pages = Vec::with_capacity(json_pages.len());

        for (i, page_verses) in json_pages.into_iter().enumerate() {
            let page_number: u16 = (i as u16) + 1;

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

            // Then convert to Rc<[Verse]>
            let verses_rc = Rc::from(verses);

            pages.push(Page::new(page_number, verses_rc));
        }

        Mushaf {
            lines_per_page: 15,
            max_page: pages.len() as u16,
            pages: Rc::from(pages),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
struct JsonVerse {
    sura: u8,
    ayah: u16,
    lines: f32,
    y: u8,
    x: f32,
}
