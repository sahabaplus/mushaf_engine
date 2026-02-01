use super::super::*;
use std::path::PathBuf;

use crate::king_fahad_mushaf::{JsonVerse, KingFahadMushaf};

fn setup_metadata() -> QuranMetadata {
    let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    data_path.push("data");
    data_path.push("king_fahad_mushaf.json");

    let mushaf = {
        let path = data_path.to_str().expect("expect `data_path` to be a valid string");
        // Load from provided JSON path
        let file_content = std::fs::read_to_string(path).expect("Failed to read mushaf data file");

        let pages: Vec<Vec<JsonVerse>> =
            serde_json::from_str(&file_content).expect("Failed to parse mushaf JSON data");

        KingFahadMushaf::create_mushaf_from_pages(pages)
    };
    QuranMetadata::from_mushaf(&mushaf)
}

#[test]
fn metadata_creation() {
    let metadata = setup_metadata();

    // Verify we have exactly 114 suras
    assert_eq!(metadata.total_suras(), 114);

    // Test Al-Fatiha info
    let fatiha = metadata.get_sura_info(1).expect("expect `fatiha` not to be Err");
    assert_eq!(fatiha.number, 1);
    assert_eq!(fatiha.total_verses, 7); // Al-Fatiha has 7 verses
    assert!(fatiha.lines_with_header > fatiha.lines); // Should include header

    // Test At-Tawbah (no bismillah)
    let tawbah = metadata.get_sura_info(9).expect("expect `tawbah` not to be Err");
    assert_eq!(tawbah.number, 9);
    assert!((tawbah.lines_with_header - (tawbah.lines + 1.0)).abs() < f32::EPSILON); // Only 1 line for header

    // Test invalid sura numbers
    assert!(metadata.get_sura_info(0).is_err());
    assert!(metadata.get_sura_info(115).is_err());
}

#[test]
fn find_sura_by_page() {
    let metadata = setup_metadata();

    // First page should contain Al-Fatiha
    let suras_on_page1 = metadata.find_sura_by_page(1);
    assert!(suras_on_page1.is_some());
    let suras_on_page1 = suras_on_page1.expect("expect `suras_on_page1` not to be None");
    assert!(suras_on_page1.contains(&1));

    // Test a page that contains multiple suras
    // This is just an example - adjust with actual data
    let page_with_suras = metadata
        .find_sura_by_page(600)
        .expect("expect `page_with_suras` not to be None");
    println!("Suras on page 600: {page_with_suras:?}");
    assert!(!page_with_suras.is_empty());
}

#[test]
fn get_lines_range() {
    let metadata = setup_metadata();

    // Test line counting for a range of suras
    let lines_1_to_3 = metadata.get_lines_range(1, 3).expect("expect `lines_1_to_3` not to be Err");
    assert!(lines_1_to_3 > 0.0, "Lines for suras 1-3 should be positive");

    // Test with invalid range
    assert!(metadata.get_lines_range(0, 115).is_err());
}

#[test]
fn t() {
    let metadata = setup_metadata();

    let suras = metadata.find_sura_by_page(106);
    assert!(suras.is_some());
    let suras = suras.expect("expect `suras` not to be None");
    assert!(suras.iter().eq([4, 5].iter()));

    let suras = metadata.find_sura_by_page(604);
    assert!(suras.is_some());
    let suras = suras.expect("expect `suras` not to be None");
    assert!(suras.iter().eq([112, 113, 114].iter()));

    let suras = metadata.find_sura_by_page(605);
    assert!(suras.is_none());
}
