use rust_quran_engine::{
    king_fahad_mushaf::{JsonVerse, KingFahadMushaf},
    mushaf_engine::{IMushafEngine, base_mushaf_engine::BaseMushafEngine},
    navigation::{Direction, NavigationSettings, VersePosition},
};

use std::{path::PathBuf, sync::Arc};

fn setup_engine() -> BaseMushafEngine {
    let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    data_path.push("data");
    data_path.push("king_fahad_mushaf.json");

    let mushaf = Arc::new({
        let path = data_path.to_str().expect("expect `data_path` to be a valid string");
        // Load from provided JSON path
        let file_content = std::fs::read_to_string(path).expect("Failed to read mushaf data file");

        let pages: Vec<Vec<JsonVerse>> =
            serde_json::from_str(&file_content).expect("Failed to parse mushaf JSON data");

        KingFahadMushaf::create_mushaf_from_pages(pages)
    });
    BaseMushafEngine::new(mushaf)
}

#[test]

fn manual_tests() {
    let engine = setup_engine();
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(103, 3))
        .lower_bound(VersePosition::end());
    let direction = Direction::Upwards;

    let result = engine.navigate(99.0 * 15.0, VersePosition::new(114, 1), direction, settings);

    assert!(result.is_ok());
    let result = result.expect("Result must be Ok");
    assert_eq!(VersePosition::new(103, 3), result.verse);
}

/// Navigate from (114,1) upwards with bounds (114,1)..(104,9), iteration_limit 9, 56 lines.
#[test]
fn manual_test_navigate_from_114_upwards_with_bounds() {
    let engine = setup_engine();

    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(104, 9))
        .lower_bound(VersePosition::new(114, 1))
        .iteration_limit(9)
        .ignore_sura_header(false);

    let direction = Direction::Upwards;
    let result = engine.navigate(57.0, VersePosition::new(114, 1), direction, settings);

    assert!(result.is_ok());
    let result = result.expect("Result must be Ok");
    println!("{}", result);
}
