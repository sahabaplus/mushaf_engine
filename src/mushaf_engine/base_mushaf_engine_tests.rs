use crate::{
    king_fahad_mushaf::{JsonVerse, KingFahadMushaf},
    mushaf_engine::{IMushafEngine, base_mushaf_engine::BaseMushafEngine},
    navigation::*,
};
use colored::Colorize;
use std::{path::PathBuf, sync::Arc};

// Test tolerance constants
/// Tolerance for line distance comparisons in tests
const LINE_TOLERANCE: f32 = 0.1;

/// Tolerance for verse number comparisons in tests (for cycle verification)
const VERSE_TOLERANCE_SMALL: i32 = 2;

/// Tolerance for verse number comparisons in tests (for excluding bounds)
const VERSE_TOLERANCE_LARGE: i32 = 3;

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
fn navigate_15_lines() {
    let engine = setup_engine();
    let lines = 15.1f32;
    let result = engine
        .navigate(
            lines,
            VersePosition::new(5, 3),
            Direction::Upwards,
            Default::default(),
        )
        .expect("expect `navigate` to succeed");
    println!(
        "{:16}{}",
        "Lines: ".bold().cyan(),
        lines.to_string().yellow().bold()
    );
    println!("{result}");
    let NavigationResult {
        distance_moved,
        end_of_page,
        end_of_sura,
        overflow,
        verse,
        ..
    } = result;

    assert!((distance_moved - 15.0).abs() < f32::EPSILON);
    assert_eq!(verse.sura, 5);
    assert_eq!(verse.number, 5);
    assert!(end_of_page.is_some());
    assert!(end_of_sura.is_some());
    assert!(overflow.is_some());
    let overflow = overflow.expect("expect `overflow` not to be None");
    let end_of_page = end_of_page.expect("expect `end_of_page` not to be None");
    let end_of_sura = end_of_sura.expect("expect `end_of_sura` not to be None");
    assert!((overflow.overflow_lines - 7.9f32).abs() < f32::EPSILON);
    assert!((end_of_page.lines_distance - -0.1).abs() < f32::EPSILON);
    assert!((end_of_sura.lines_distance - 300.0).abs() < f32::EPSILON);
}

#[test]
fn navigation_overflow() {
    let engine = setup_engine();

    let v = engine
        .navigate(
            14.7f32,
            VersePosition::new(114, 1),
            Direction::Upwards,
            NavigationSettings::builder(),
        )
        .expect("expect `navigate` to succeed");

    println!("{v}");
    assert!(v.overflow.is_some());
    assert!(v.end_of_sura.is_some());
    assert_eq!(
        v.overflow.expect("expect `overflow` not to be None").overflowed_verse,
        v.end_of_sura.expect("expect `end_of_sura` not to be None").last_verse
    );
    assert_eq!(v.verse.sura, 112);
    assert_eq!(v.verse.number, 3);
}

#[test]
fn navigate_zero_lines() {
    let engine = setup_engine();

    // First verse of Al-Fatiha
    let result = engine
        .navigate(
            0.0,
            VersePosition::start(),
            Direction::Downwards,
            Default::default(),
        )
        .expect("expect `navigate` to succeed");
    assert_eq!(result.verse.sura, 1);
    assert_eq!(result.verse.number, 1);
}

#[test]
fn calculating_lines() {
    let engine = setup_engine();
    let lines = engine.calculate_lines(
        VersePosition::start(),
        VersePosition::end(),
        Direction::Downwards,
        Default::default(),
    );
    assert!(lines.is_ok());
}

#[test]
fn multiple_iterations() {
    let engine = setup_engine();
    let settings = NavigationSettings::builder().ignore_sura_header(true);
    let lines = engine.calculate_lines(
        VersePosition::start(),
        VersePosition::end(),
        Direction::Downwards,
        settings,
    );
    assert!(lines.is_ok());
    println!("{lines:?}");
    let whole_mushaf_lines = lines.expect("expect `lines` not to be None");

    let navigate_by_whole_mushaf_ratio = |ratio: f32, additional_lines: f32| {
        engine
            .navigate(
                whole_mushaf_lines * ratio + additional_lines,
                VersePosition::start(),
                Direction::Downwards,
                settings.iteration_limit(1),
            )
            .expect("expect `navigate` to succeed")
    };

    // One full cycle
    let result = navigate_by_whole_mushaf_ratio(1.0, 0.0);
    assert_eq!(VersePosition::end(), result.verse);

    // One full cycle plus small overflow
    let result = navigate_by_whole_mushaf_ratio(1.0, 0.1);
    assert_eq!(VersePosition::end(), result.verse);
    assert!(result.overflow.is_some());
    let overflow = result.overflow.expect("expect `overflow` not to be None");
    // Should round back to start
    assert_eq!(VersePosition::start(), overflow.overflowed_verse);

    // One and a half cycle
    let result = navigate_by_whole_mushaf_ratio(1.5, 0.0);
    assert_eq!(result.verse.sura, 18); // Surat Al-Kahf
    assert!((30..=39).contains(&result.verse.number));

    // Two full cycles
    let result = navigate_by_whole_mushaf_ratio(2.0, 0.0);
    assert_eq!(VersePosition::end(), result.verse);

    // Two full cycles plus small overflow
    let result = navigate_by_whole_mushaf_ratio(2.0, 0.1);
    assert_eq!(VersePosition::end(), result.verse);
    assert!(result.end_of_page.is_some());
    assert!(result.end_of_sura.is_some());
    let end_of_page = result.end_of_page.expect("expect `end_of_page` not to be None");
    let end_of_sura = result.end_of_sura.expect("expect `end_of_sura` not to be None");
    // Stick at the end
    assert_eq!(VersePosition::end(), end_of_page.last_verse);
    assert_eq!(VersePosition::end(), end_of_sura.last_verse);
}

#[test]
fn bounded_navigation() {
    let engine = setup_engine();
    let start_bound = VersePosition::new(2, 1);
    let end_bound = VersePosition::new(2, 50);
    let settings = NavigationSettings::builder().upper_bound(start_bound).lower_bound(end_bound);
    let result = engine.navigate(10.0, VersePosition::start(), Default::default(), settings);
    assert!(result.is_err());
    let err = result.expect_err("expect `err` not to be None");
    assert!(err == NavigationError::OutOfBounds);

    let result = engine.navigate(0.0, start_bound, Default::default(), settings);
    assert!(result.is_ok());
    let verse = result.expect("expect `result` not to be None").verse;
    assert_eq!(start_bound, verse);

    // Massive overflow
    let result = engine.navigate(1000.0, start_bound, Default::default(), settings);
    assert!(result.is_ok());
    let verse = result.expect("expect `result` not to be None").verse;
    assert_eq!(end_bound, verse);

    // Multiple iterations
    let bounds_lines = engine.calculate_lines(start_bound, end_bound, Default::default(), settings);
    assert!(bounds_lines.is_ok());
    let bounds_lines = bounds_lines.expect("expect `bounds_lines` not to be None");
    let result = engine.navigate(
        bounds_lines * 2.0 + 15.0,
        start_bound,
        Default::default(),
        settings.iteration_limit(2),
    );
    assert!(result.is_ok());
    let verse = result.expect("expect `result` not to be None").verse;
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 10);
}

#[test]
fn calculate_lines_with_bounds() {
    let engine = setup_engine();
    let start_bound = VersePosition::new(2, 1);
    let end_bound = VersePosition::new(2, 286);
    let settings = NavigationSettings::builder()
        .iteration_limit(2)
        .upper_bound(start_bound)
        .lower_bound(end_bound);

    let full_sura_lines_metadata = engine
        .quran_metadata
        .get_sura_info(2)
        .expect("expect `full_sura` not to be None")
        .lines_with_header;
    let full_sura_in_normal_direction = engine.calculate_lines(
        VersePosition::new(2, 1),
        VersePosition::new(2, 286),
        Direction::Downwards,
        Default::default(),
    );
    let lines = engine.calculate_lines(
        VersePosition::new(2, 280),
        VersePosition::new(2, 279),
        Direction::Downwards,
        settings,
    );
    assert!(full_sura_in_normal_direction.is_ok());
    assert!(lines.is_ok());

    let full_sura_in_normal_direction = full_sura_in_normal_direction
        .expect("expect `full_sura_in_normal_direction` not to be None");
    let lines = lines.expect("expect `lines` not to be None");

    // They should be the same
    assert_eq!(full_sura_in_normal_direction, lines);
    assert_eq!(
        full_sura_in_normal_direction,
        (full_sura_lines_metadata * 10.0).round() / 10.0
    );
}

#[test]
fn calculate_lines_with_excluding_bounds() {
    let engine = setup_engine();

    // Test 1: Exclude verses 10-20 in sura 2 (exclusive bounds) - Downwards direction
    // Excluding bounds: lower_bound=(2,10), upper_bound=(2,20)
    // This means: exclude verses strictly between (2,10) and (2,20), i.e., verses 11-19
    let excluding_settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(2, 20)) // upper > lower = excluding mode
        .lower_bound(VersePosition::new(2, 10));

    // Calculate lines from (2,1) to (2,30) with excluding bounds
    // Expected: include verses 1-10, skip 11-19, include 20-30
    let lines_with_excluding = engine
        .calculate_lines(
            VersePosition::new(2, 1),
            VersePosition::new(2, 30),
            Direction::Downwards,
            excluding_settings,
        )
        .expect("Should calculate lines");

    // Directly calculate what should be included: verses 1-10 and 20-30
    let lines_1_to_10 = engine
        .calculate_lines(
            VersePosition::new(2, 1),
            VersePosition::new(2, 10),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should calculate");

    let lines_20_to_30 = engine
        .calculate_lines(
            VersePosition::new(2, 20),
            VersePosition::new(2, 30),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should calculate");

    let expected_lines = lines_1_to_10 + lines_20_to_30;
    assert!(
        (lines_with_excluding - expected_lines).abs() < LINE_TOLERANCE,
        "Downwards: Expected {} lines (verses 1-10 + 20-30), got {} lines",
        expected_lines,
        lines_with_excluding
    );

    // Test 2: Downwards with different range
    // Test excluding verses 50-100 in sura 2
    let excluding_settings_2 = NavigationSettings::builder()
        .upper_bound(VersePosition::new(2, 100))
        .lower_bound(VersePosition::new(2, 50));

    let lines_with_excluding_2 = engine
        .calculate_lines(
            VersePosition::new(2, 1),
            VersePosition::new(2, 150),
            Direction::Downwards,
            excluding_settings_2,
        )
        .expect("Should calculate lines");

    // Calculate what should be included: verses 1-50 and 100-150
    let lines_1_to_50 = engine
        .calculate_lines(
            VersePosition::new(2, 1),
            VersePosition::new(2, 50),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should calculate");

    let lines_100_to_150 = engine
        .calculate_lines(
            VersePosition::new(2, 100),
            VersePosition::new(2, 150),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should calculate");

    let expected_lines_2 = lines_1_to_50 + lines_100_to_150;
    assert!(
        (lines_with_excluding_2 - expected_lines_2).abs() < LINE_TOLERANCE,
        "Downwards 2: Expected {} lines (verses 1-50 + 100-150), got {} lines",
        expected_lines_2,
        lines_with_excluding_2
    );

    // Test 3: Cross-sura excluding bounds - Downwards direction
    // Exclude verses between (1,5) and (2,5) - should skip verses 1:6-7 and 2:1-4
    let cross_sura_excluding = NavigationSettings::builder()
        .upper_bound(VersePosition::new(2, 5))
        .lower_bound(VersePosition::new(1, 5));

    // Calculate from (1,1) to (2,10) with excluding bounds
    // Expected: include (1,1-5), skip (1,6-7) and (2,1-4), include (2,5-10)
    let lines_cross_sura_down = engine
        .calculate_lines(
            VersePosition::new(1, 1),
            VersePosition::new(2, 10),
            Direction::Downwards,
            cross_sura_excluding,
        )
        .expect("Should calculate lines");

    // Directly calculate what should be included
    let sura1_included = engine
        .calculate_lines(
            VersePosition::new(1, 1),
            VersePosition::new(1, 5),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should calculate");

    let sura2_included = engine
        .calculate_lines(
            VersePosition::new(2, 5),
            VersePosition::new(2, 10),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should calculate");

    let expected_cross_sura_down = sura1_included + sura2_included;
    assert!(
        (lines_cross_sura_down - expected_cross_sura_down).abs() < LINE_TOLERANCE,
        "Cross-sura Downwards: Expected {} lines ((1,1-5) + (2,5-10)), got {} lines",
        expected_cross_sura_down,
        lines_cross_sura_down
    );

    // Test 4: Another cross-sura excluding test
    // Exclude verses between (2,100) and (3,50)
    let cross_sura_excluding_2 = NavigationSettings::builder()
        .upper_bound(VersePosition::new(3, 50))
        .lower_bound(VersePosition::new(2, 100));

    let lines_cross_sura_2 = engine
        .calculate_lines(
            VersePosition::new(2, 50),
            VersePosition::new(3, 100),
            Direction::Downwards,
            cross_sura_excluding_2,
        )
        .expect("Should calculate lines");

    // Calculate what should be included: (2,50-100) + (3,50-100)
    let sura2_part = engine
        .calculate_lines(
            VersePosition::new(2, 50),
            VersePosition::new(2, 100),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should calculate");

    let sura3_part = engine
        .calculate_lines(
            VersePosition::new(3, 50),
            VersePosition::new(3, 100),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should calculate");

    let expected_cross_sura_2 = sura2_part + sura3_part;
    assert!(
        (lines_cross_sura_2 - expected_cross_sura_2).abs() < LINE_TOLERANCE,
        "Cross-sura 2: Expected {} lines ((2,50-100) + (3,50-100)), got {} lines",
        expected_cross_sura_2,
        lines_cross_sura_2
    );

    // Test 5: Verify excluded verses are actually skipped
    // For cross-sura excluding, verify that excluded range has different line count
    let excluded_range_down = engine
        .calculate_lines(
            VersePosition::new(1, 6),
            VersePosition::new(2, 4),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should calculate");

    // The excluded range should NOT be included in the result
    let full_range_down = engine
        .calculate_lines(
            VersePosition::new(1, 1),
            VersePosition::new(2, 10),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should calculate");

    // With excluding, we should have: full_range - excluded_range
    let expected_with_excluding = full_range_down - excluded_range_down;
    assert!(
        (lines_cross_sura_down - expected_with_excluding).abs() < LINE_TOLERANCE,
        "Verification: Expected {} lines (full - excluded), got {} lines",
        expected_with_excluding,
        lines_cross_sura_down
    );
}

#[test]
fn test_navigation_with_boundary_crossing() {
    let engine = setup_engine();

    // Navigate with excluding bounds in upward direction with iteration cycling
    // This will cause wrapping from sura 1 to sura 114 when cycling occurs
    let excluding_settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(78, 1))
        .lower_bound(VersePosition::new(2, 286));

    let result = engine
        .navigate(
            1600.0,
            VersePosition::new(2, 1),
            Direction::Downwards,
            excluding_settings,
        )
        .expect("Should navigate successfully");

    println!("\n=== Navigation Result with Boundary Crossing ===");
    println!("{result}");
    println!("================================================\n");

    // With excluding bounds and no iteration_limit, navigation skips the excluded range
    // but doesn't cycle back to the start, so crossed_boundaries should be false.
    // Per the new semantics, crossed_boundaries is only true when cycling (wrapping around
    // and returning to or passing through the initial verse), not when simply skipping
    // an excluded range.
    assert!(
        !result.cycle_info.crossed_boundaries(),
        "Navigation should not have crossed_boundaries without cycling"
    );

    // The verse should be somewhere in the higher suras (78-114) after skipping excluded range
    assert!(
        result.verse.sura >= 78,
        "After skipping excluded range, should be in higher suras, got sura {}",
        result.verse.sura
    );
}

// ==================== crossed_boundaries tests ====================
//
// crossed_boundaries is ONLY true when navigation wraps from end_bound
// back to start_bound (via iteration cycling). It is NOT true when:
// - Navigation simply reaches the end and stops
// - Navigation skips an excluded range
// - Navigation runs out of lines mid-range

#[test]
fn test_crossed_boundaries_false_when_stopping_at_end_downwards() {
    let engine = setup_engine();

    // Navigate within a bounded range without iteration_limit.
    // Navigation should stop at end_bound, NOT set crossed_boundaries.
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(1, 1))
        .lower_bound(VersePosition::new(1, 7));

    let result = engine
        .navigate(
            1000.0, // More than enough to reach end
            VersePosition::new(1, 1),
            Direction::Downwards,
            settings,
        )
        .expect("Should navigate successfully");

    assert_eq!(VersePosition::new(1, 7), result.verse);
    assert!(
        !result.cycle_info.crossed_boundaries(),
        "Stopping at end_bound should NOT set crossed_boundaries"
    );
    assert_eq!(result.cycle_info.cycles_completed(), 0);
}

#[test]
fn test_crossed_boundaries_false_when_stopping_at_end_upwards() {
    let engine = setup_engine();

    // Upward navigation with bounds, no cycling. Should stop at upper_bound sura.
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(112, 1))
        .lower_bound(VersePosition::new(114, 6));

    let result = engine
        .navigate(
            1000.0, // More than enough to reach end
            VersePosition::new(114, 1),
            Direction::Upwards,
            settings,
        )
        .expect("Should navigate successfully");

    assert_eq!(result.verse.sura, 112);
    assert!(
        !result.cycle_info.crossed_boundaries(),
        "Stopping at end_bound (upwards) should NOT set crossed_boundaries"
    );
    assert_eq!(result.cycle_info.cycles_completed(), 0);
}

#[test]
fn test_crossed_boundaries_false_when_lines_run_out() {
    let engine = setup_engine();

    // Navigate a small number of lines — not enough to reach the end.
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(1, 1))
        .lower_bound(VersePosition::new(10, 109));

    let result = engine
        .navigate(
            5.0,
            VersePosition::new(1, 1),
            Direction::Downwards,
            settings,
        )
        .expect("Should navigate successfully");

    assert!(
        !result.cycle_info.crossed_boundaries(),
        "Running out of lines mid-range should NOT set crossed_boundaries"
    );
    assert_eq!(result.cycle_info.cycles_completed(), 0);
}

#[test]
fn test_crossed_boundaries_true_when_cycling_downwards() {
    let engine = setup_engine();

    // Bounded range with iteration_limit allowing cycling.
    // Navigation MUST wrap from end_bound back to start_bound.
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(1, 1))
        .lower_bound(VersePosition::new(1, 7))
        .iteration_limit(2);

    let result = engine
        .navigate(
            100.0, // Enough to cycle through Al-Fatiha multiple times
            VersePosition::new(1, 1),
            Direction::Downwards,
            settings,
        )
        .expect("Should navigate successfully");

    assert!(
        result.cycle_info.cycles_completed() > 0,
        "Should have completed at least one cycle"
    );
    assert!(
        result.cycle_info.crossed_boundaries(),
        "Wrapping from end_bound to start_bound MUST set crossed_boundaries"
    );
}

#[test]
fn test_crossed_boundaries_true_when_cycling_upwards() {
    let engine = setup_engine();

    // Upward navigation with cycling through a small sura range.
    // Start from (114, 6) — the start_bound for upward direction.
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(112, 1))
        .lower_bound(VersePosition::new(114, 6))
        .iteration_limit(2);

    let result = engine
        .navigate(
            100.0, // Enough to cycle
            VersePosition::new(114, 6),
            Direction::Upwards,
            settings,
        )
        .expect("Should navigate successfully");

    assert!(
        result.cycle_info.cycles_completed() > 0,
        "Should have completed at least one cycle"
    );
    assert!(
        result.cycle_info.crossed_boundaries(),
        "Wrapping from end_bound to start_bound (upwards) MUST set crossed_boundaries"
    );
}

#[test]
fn test_crossed_boundaries_false_with_excluding_bounds() {
    let engine = setup_engine();

    // Excluding bounds: skip a range but do NOT cycle.
    // crossed_boundaries should remain false.
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(2, 20))
        .lower_bound(VersePosition::new(2, 10));

    let result = engine
        .navigate(
            30.0,
            VersePosition::new(2, 1),
            Direction::Downwards,
            settings,
        )
        .expect("Should navigate successfully");

    assert!(
        !result.cycle_info.crossed_boundaries(),
        "Skipping an excluded range should NOT set crossed_boundaries"
    );
}

#[test]
fn test_crossed_boundaries_false_full_quran_no_cycle() {
    let engine = setup_engine();

    // Full Quran bounds, no iteration. Navigate a small distance.
    let result = engine
        .navigate(
            30.0,
            VersePosition::new(50, 1),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should navigate successfully");

    assert!(
        !result.cycle_info.crossed_boundaries(),
        "Normal navigation through full Quran should NOT set crossed_boundaries"
    );
    assert_eq!(result.cycle_info.cycles_completed(), 0);
}

#[test]
fn test_crossed_boundaries_exactly_at_end_no_cross() {
    let engine = setup_engine();

    // Navigate exactly to the end_bound. No cycling should occur.
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(1, 1))
        .lower_bound(VersePosition::new(1, 7));

    // Calculate exact lines for the range
    let exact_lines = engine
        .calculate_lines(
            VersePosition::new(1, 1),
            VersePosition::new(1, 7),
            Direction::Downwards,
            settings,
        )
        .expect("Should calculate lines");

    let result = engine
        .navigate(
            exact_lines,
            VersePosition::new(1, 1),
            Direction::Downwards,
            settings,
        )
        .expect("Should navigate successfully");

    assert_eq!(VersePosition::new(1, 7), result.verse);
    assert!(
        !result.cycle_info.crossed_boundaries(),
        "Landing exactly on end_bound should NOT set crossed_boundaries"
    );
    assert_eq!(result.cycle_info.cycles_completed(), 0);
}

#[test]
fn test_crossed_boundaries_single_cycle_then_stop() {
    let engine = setup_engine();

    // iteration_limit=1: one wrap, then stop at end.
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(1, 1))
        .lower_bound(VersePosition::new(1, 7))
        .iteration_limit(1);

    // Calculate exact lines for one full cycle (end_bound → start_bound → end_bound)
    let cycle_lines = engine
        .calculate_lines(
            VersePosition::new(1, 1),
            VersePosition::new(1, 7),
            Direction::Downwards,
            settings,
        )
        .expect("Should calculate lines");

    // Navigate slightly more than one cycle to force wrapping
    let result = engine
        .navigate(
            cycle_lines + 1.0,
            VersePosition::new(1, 1),
            Direction::Downwards,
            settings,
        )
        .expect("Should navigate successfully");

    assert!(
        result.cycle_info.crossed_boundaries(),
        "After wrapping once, crossed_boundaries MUST be true"
    );
    assert!(
        result.cycle_info.cycles_completed() >= 1,
        "Should have completed at least 1 cycle"
    );
}

#[test]
fn test_navigation_with_cycles() {
    let engine = setup_engine();

    // Navigate with a small bounded range and multiple iterations
    let settings_with_cycles = NavigationSettings::builder()
        .upper_bound(VersePosition::new(1, 1))
        .lower_bound(VersePosition::new(1, 7))
        .iteration_limit(3);

    // Navigate through Al-Fatiha multiple times
    let result = engine
        .navigate(
            50.0, // Enough to cycle multiple times
            VersePosition::new(1, 1),
            Direction::Downwards,
            settings_with_cycles,
        )
        .expect("Should navigate successfully");

    println!("\n=== Navigation Result with Cycles ===");
    println!("{result}");
    println!("======================================\n");

    // Verify cycles were tracked
    assert!(
        result.cycle_info.cycles_completed() > 0,
        "Should complete at least one cycle, got {}",
        result.cycle_info.cycles_completed()
    );

    // This helps the caller understand that we went through the same range multiple times
    println!(
        "Completed {} cycles through Al-Fatiha (verses 1-7)",
        result.cycle_info.cycles_completed()
    );
}

#[test]
fn test_inverse_relationship_navigate_to_calculate_lines() {
    let engine = setup_engine();

    // Test: navigate → calculate_lines with cycle_distance
    // Navigate from start to some position, then verify we can reconstruct the distance
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(1, 1))
        .lower_bound(VersePosition::new(1, 7))
        .iteration_limit(5);

    let start = VersePosition::new(1, 1);
    let lines_to_navigate = 25.0;

    let result = engine
        .navigate(lines_to_navigate, start, Direction::Downwards, settings)
        .expect("Should navigate successfully");

    // Now calculate direct distance from start to end
    let direct_distance = engine
        .calculate_lines(start, &result.verse, Direction::Downwards, settings)
        .expect("Should calculate lines");

    // Reconstruct total distance using the formula:
    // When cycles_completed > 0, the formula is:
    // distance = (cycles_completed - 1) * cycle_distance + direct_distance
    //
    // This works because:
    // - cycles_completed counts passes through the start verse
    // - Each pass adds one cycle_distance worth of lines
    // - The current (partial or complete) cycle contributes direct_distance
    // - So we have (N-1) complete previous cycles + current cycle
    let reconstructed_distance = result.cycle_info.reconstruct_distance(direct_distance);

    println!(
        "Navigate: distance_moved={}, cycles={}, cycle_distance={}, direct={}, reconstructed={}, diff={}",
        result.distance_moved,
        result.cycle_info.cycles_completed(),
        result.cycle_info.cycle_distance(),
        direct_distance,
        reconstructed_distance,
        (reconstructed_distance - result.distance_moved).abs()
    );

    // Verify the reconstructed distance matches distance_moved
    let diff = (reconstructed_distance - result.distance_moved).abs();
    assert!(
        diff < 0.1,
        "Reconstructed distance ({}) should match distance_moved ({}), diff={}",
        reconstructed_distance,
        result.distance_moved,
        diff
    );

    // Verify that if we navigate with the reconstructed distance, we get the same result
    let verify_result = engine
        .navigate(
            reconstructed_distance,
            start,
            Direction::Downwards,
            settings,
        )
        .expect("Should navigate successfully");

    // Should reach the same or very close verse
    assert_eq!(
        verify_result.verse.sura, result.verse.sura,
        "Should reach same sura"
    );
    assert!(
        ((verify_result.verse.number as i32) - (result.verse.number as i32)).abs()
            <= VERSE_TOLERANCE_SMALL,
        "Should reach same or nearby verse, got {} vs {}",
        verify_result.verse.number,
        result.verse.number
    );
}

#[test]
fn test_inverse_relationship_calculate_lines_to_navigate() {
    let engine = setup_engine();

    // Test: Using cycle_distance to navigate with cycles
    // First navigate to discover cycle_distance, then use it to navigate precisely
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(2, 1))
        .lower_bound(VersePosition::new(2, 50))
        .iteration_limit(3);

    let start = VersePosition::new(2, 1);
    let end = VersePosition::new(2, 30);

    // First, do a navigation to discover the cycle_distance
    let discovery_result = engine
        .navigate(100.0, start, Direction::Downwards, settings)
        .expect("Should navigate successfully");

    // Only proceed if we actually completed at least one cycle
    if discovery_result.cycle_info.cycles_completed() > 0
        && discovery_result.cycle_info.cycle_distance() > 0.0
    {
        println!(
            "Discovered cycle_distance: {}",
            discovery_result.cycle_info.cycle_distance()
        );

        // Now calculate direct distance from start to end
        let direct_distance = engine
            .calculate_lines(start, end, Direction::Downwards, settings)
            .expect("Should calculate lines");

        // Compute total lines for 1 cycle + direct distance to end
        let desired_cycles = 1;
        let total_lines = (desired_cycles as f32) * discovery_result.cycle_info.cycle_distance()
            + direct_distance;

        // Navigate with this computed distance
        let result = engine
            .navigate(total_lines, start, Direction::Downwards, settings)
            .expect("Should navigate successfully");

        // Verify we reached the end position (or very close, due to rounding)
        assert_eq!(
            result.verse.sura,
            end.sura(),
            "Navigation should reach same sura, got {}",
            result.verse.sura
        );
        assert!(
            ((result.verse.number as i32) - (end.verse() as i32)).abs() <= VERSE_TOLERANCE_LARGE,
            "Navigation should reach end position or nearby, got ({},{}) vs ({},{})",
            result.verse.sura,
            result.verse.number,
            end.sura(),
            end.verse()
        );

        // Verify we completed approximately the desired number of cycles
        assert!(
            result.cycle_info.cycles_completed() >= desired_cycles
                || result.cycle_info.crossed_boundaries(),
            "Should complete at least {} cycles, got {}",
            desired_cycles,
            result.cycle_info.cycles_completed()
        );
    }
}

#[test]
fn test_inverse_relationship_with_excluding_bounds() {
    let engine = setup_engine();

    // Test inverse relationship with excluding bounds using cycle_distance
    let excluding_settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(2, 20))
        .lower_bound(VersePosition::new(2, 10));

    let start = VersePosition::new(2, 1);
    let lines_to_navigate = 15.0;

    let result = engine
        .navigate(
            lines_to_navigate,
            start,
            Direction::Downwards,
            excluding_settings,
        )
        .expect("Should navigate successfully");

    // Calculate direct distance from start to result
    let direct_distance = engine
        .calculate_lines(
            start,
            &result.verse,
            Direction::Downwards,
            excluding_settings,
        )
        .expect("Should calculate lines");

    // Reconstruct total distance using the formula
    let reconstructed_distance = result.cycle_info.reconstruct_distance(direct_distance);

    assert!(
        (reconstructed_distance - result.distance_moved).abs() < LINE_TOLERANCE,
        "With excluding bounds: Reconstructed ({}) should match distance_moved ({})",
        reconstructed_distance,
        result.distance_moved
    );
}

#[test]
#[ignore] // Manual debugging test with no assertions
fn manual_free_tests() {
    let engine = setup_engine();

    let result = engine
        .navigate(
            100.0,
            VersePosition::new(1, 1),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should navigate successfully");
    println!("{result}");

    let result = engine
        .navigate(
            100.0,
            VersePosition::new(1, 1),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should navigate successfully");
    println!("{result}");

    let result = engine
        .navigate(
            100.0,
            VersePosition::new(1, 1),
            Direction::Downwards,
            Default::default(),
        )
        .expect("Should navigate successfully");
    println!("{result}");
}

// ==================== previous_verse tests ====================

#[test]
fn test_previous_verse_basic() {
    let engine = setup_engine();

    // Test basic previous verse navigation
    let verse = engine.previous_verse(
        VersePosition::new(2, 5),
        Direction::Downwards,
        Default::default(),
    );
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 4);
}

#[test]
fn test_previous_verse_cross_sura() {
    let engine = setup_engine();

    // Test crossing sura boundary backwards
    let verse = engine.previous_verse(
        VersePosition::new(2, 1),
        Direction::Downwards,
        Default::default(),
    );
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 7);
}

#[test]
fn test_previous_verse_at_start_of_quran_no_wrap() {
    let engine = setup_engine();

    // At start of Quran with default settings, should return None
    let verse = engine.previous_verse(
        VersePosition::start(),
        Direction::Downwards,
        Default::default(),
    );
    assert!(
        verse.is_none(),
        "At start of Quran with default bounds, previous should return None"
    );
}

#[test]
fn test_previous_verse_at_start_with_iteration_cycles_to_end() {
    let engine = setup_engine();

    // With iteration_limit, at start should cycle to end
    let settings = NavigationSettings::builder().iteration_limit(1);

    let verse = engine.previous_verse(VersePosition::start(), Direction::Downwards, settings);
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 114);
    assert_eq!(verse.number, 6); // End of Quran
}

#[test]
fn test_previous_verse_upward_direction() {
    let engine = setup_engine();

    // In upward direction, previous goes to next sura's end
    let verse = engine.previous_verse(
        VersePosition::new(2, 1),
        Direction::Upwards,
        Default::default(),
    );
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 3);
    assert_eq!(verse.number, 200); // Sura 3 has 200 verses
}

#[test]
fn test_next_and_previous_verse_inverse() {
    let engine = setup_engine();
    let start = VersePosition::new(5, 50);

    // Get next verse
    let next = engine
        .next_verse(start, Direction::Downwards, Default::default())
        .expect("Should have next verse");
    assert_eq!(next.sura, 5);
    assert_eq!(next.number, 51);

    // Get previous of the next should return to start
    let prev = engine
        .previous_verse(&next, Direction::Downwards, Default::default())
        .expect("Should have previous verse");
    assert_eq!(prev.sura, start.sura());
    assert_eq!(prev.number, start.verse());
}

#[test]
fn test_previous_and_next_verse_inverse() {
    let engine = setup_engine();
    let start = VersePosition::new(5, 50);

    // Get previous verse
    let prev = engine
        .previous_verse(start, Direction::Downwards, Default::default())
        .expect("Should have previous verse");
    assert_eq!(prev.sura, 5);
    assert_eq!(prev.number, 49);

    // Get next of the previous should return to start
    let next = engine
        .next_verse(&prev, Direction::Downwards, Default::default())
        .expect("Should have next verse");
    assert_eq!(next.sura, start.sura());
    assert_eq!(next.number, start.verse());
}

#[test]
fn test_previous_verse_with_bounds() {
    let engine = setup_engine();
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(2, 1))
        .lower_bound(VersePosition::new(2, 50))
        .iteration_limit(0);

    // At start bound without iteration, should return None
    let verse = engine.previous_verse(VersePosition::new(2, 1), Direction::Downwards, settings);
    assert!(verse.is_none());

    // Not at start bound, should work
    let verse = engine.previous_verse(VersePosition::new(2, 10), Direction::Downwards, settings);
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 9);
}

#[test]
fn test_previous_verse_with_bounds_and_iteration() {
    let engine = setup_engine();
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(1, 1))
        .lower_bound(VersePosition::new(1, 7))
        .iteration_limit(1);

    // At start bound with iteration, should cycle to end bound
    let verse = engine.previous_verse(VersePosition::new(1, 1), Direction::Downwards, settings);
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 7); // Cycled to end bound
}

#[test]
fn test_previous_verse_excluding_mode() {
    let engine = setup_engine();
    // Exclude verses 11-19 of Sura 2
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(2, 20)) // upper > lower = excluding mode
        .lower_bound(VersePosition::new(2, 10));

    // From verse 20, previous should skip to verse 10
    let verse = engine.previous_verse(VersePosition::new(2, 20), Direction::Downwards, settings);
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 10); // Skipped 11-19
}

#[test]
fn test_previous_verse_comprehensive_traversal() {
    let engine = setup_engine();

    // Start from end of Quran and traverse backwards
    let mut current = VersePosition::end();
    let mut count = 0;
    let max_iterations = 100;

    while count < max_iterations {
        let verse = engine.previous_verse(current, Direction::Downwards, Default::default());
        if verse.is_none() {
            break;
        }
        let verse = verse.expect("expect verse");

        // Verify we're going backwards
        let current_pos = VersePosition::new(verse.sura, verse.number);
        assert!(
            current_pos < current,
            "Should be going backwards: {:?} < {:?}",
            current_pos,
            current
        );

        current = current_pos;
        count += 1;
    }

    // Should have traversed multiple verses
    assert!(
        count == max_iterations,
        "Should traverse {} verses",
        max_iterations
    );
}

// ── Reverse navigation (ported from mushaf-engine tests/reverse-navigation.test.ts) ──

#[test]
fn reverse_navigate_zero_lines_returns_start() {
    let engine = setup_engine();
    let from = VersePosition::new(5, 3);
    let result = engine
        .reverse_navigate(0.0, from, Direction::Upwards, Default::default())
        .expect("reverse_navigate");
    assert_eq!(result.verse.sura, 5);
    assert_eq!(result.verse.number, 3);
    assert_eq!(result.distance_moved, 0.0);
}

#[test]
fn reverse_navigate_negative_lines_errors() {
    let engine = setup_engine();
    let err = engine
        .reverse_navigate(
            -1.0,
            VersePosition::start(),
            Direction::Downwards,
            Default::default(),
        )
        .expect_err("Should error");
    assert!(matches!(err, NavigationError::NegativeLines));
}

#[test]
fn reverse_navigate_differs_from_navigate() {
    let engine = setup_engine();
    let from = VersePosition::new(2, 100);
    let lines = 10.0;
    let settings = NavigationSettings::default();

    let forward = engine.navigate(lines, from, Direction::Downwards, settings).expect("navigate");
    let reverse = engine
        .reverse_navigate(lines, from, Direction::Downwards, settings)
        .expect("reverse_navigate");

    assert_eq!(forward.verse.sura, 2);
    assert_eq!(forward.verse.number, 101);
    assert_eq!(reverse.verse.sura, 2);
    assert_eq!(reverse.verse.number, 96);
}

#[test]
fn reverse_navigate_from_last_of_fatiha_both_directions() {
    let engine = setup_engine();
    let from = VersePosition::new(1, 7);
    let settings = NavigationSettings::default();

    let upwards = engine
        .reverse_navigate(10.0, from, Direction::Upwards, settings)
        .expect("upwards");
    let downwards = engine
        .reverse_navigate(10.0, from, Direction::Downwards, settings)
        .expect("downwards");

    assert_eq!(upwards.verse.sura, 1);
    assert_eq!(upwards.verse.number, 1);
    assert_eq!(downwards.verse.sura, 1);
    assert_eq!(downwards.verse.number, 1);
}

#[test]
fn reverse_navigate_steps_to_earlier_verses_downwards() {
    let engine = setup_engine();
    let result = engine
        .reverse_navigate(
            15.0,
            VersePosition::new(2, 10),
            Direction::Downwards,
            Default::default(),
        )
        .expect("reverse_navigate");
    assert_eq!(result.verse.sura, 2);
    assert_eq!(result.verse.number, 1);
    assert!((result.distance_moved - 14.8).abs() < 0.15);
}

#[test]
fn reverse_navigate_cross_sura_from_78_1() {
    let engine = setup_engine();
    let from = VersePosition::new(78, 1);
    let settings = NavigationSettings::default();

    let upwards = engine
        .reverse_navigate(10.0, from, Direction::Upwards, settings)
        .expect("upwards");
    let downwards = engine
        .reverse_navigate(10.0, from, Direction::Downwards, settings)
        .expect("downwards");

    assert_eq!(upwards.verse.sura, 79);
    assert_eq!(downwards.verse.sura, 77);
}

#[test]
fn reverse_navigate_upward_from_middle_crosses_next_sura() {
    let engine = setup_engine();
    let result = engine
        .reverse_navigate(
            20.0,
            VersePosition::new(2, 5),
            Direction::Upwards,
            Default::default(),
        )
        .expect("reverse_navigate");
    assert_eq!(result.verse.sura, 3);
    assert!(result.verse.number > 1);
}

#[test]
fn reverse_then_forward_returns_to_start() {
    let engine = setup_engine();
    let start = VersePosition::new(2, 50);
    let lines = 20.0;
    let settings = NavigationSettings::default();

    let after_reverse = engine
        .reverse_navigate(lines, start, Direction::Downwards, settings)
        .expect("reverse");
    let after_forward = engine
        .navigate(
            lines,
            VersePosition::new(after_reverse.verse.sura, after_reverse.verse.number),
            Direction::Downwards,
            settings,
        )
        .expect("forward");

    assert_eq!(after_reverse.verse.sura, 2);
    assert_eq!(after_reverse.verse.number, 37);
    assert_eq!(after_forward.verse.sura, 2);
    assert_eq!(after_forward.verse.number, 50);
}

#[test]
fn forward_then_reverse_lands_near_start() {
    let engine = setup_engine();
    let start = VersePosition::new(2, 50);
    let lines = 20.0;
    let settings = NavigationSettings::default();

    let after_forward =
        engine.navigate(lines, start, Direction::Downwards, settings).expect("forward");
    let after_reverse = engine
        .reverse_navigate(
            lines,
            VersePosition::new(after_forward.verse.sura, after_forward.verse.number),
            Direction::Downwards,
            settings,
        )
        .expect("reverse");

    assert_eq!(after_forward.verse.sura, 2);
    assert_eq!(after_forward.verse.number, 59);
    assert_eq!(after_reverse.verse.sura, 2);
    assert_eq!(after_reverse.verse.number, 49);
}

#[test]
fn reverse_navigate_respects_bounds() {
    let engine = setup_engine();
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(2, 1))
        .lower_bound(VersePosition::new(2, 10));

    let result = engine
        .reverse_navigate(
            5.0,
            VersePosition::new(2, 5),
            Direction::Downwards,
            settings,
        )
        .expect("reverse_navigate");
    assert_eq!(result.verse.sura, 2);
    assert_eq!(result.verse.number, 2);
}

#[test]
fn reverse_navigate_cycles_within_fatiha() {
    let engine = setup_engine();
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(1, 1))
        .lower_bound(VersePosition::new(1, 7))
        .iteration_limit(2);

    let reverse = engine
        .reverse_navigate(
            32.0,
            VersePosition::new(1, 4),
            Direction::Downwards,
            settings,
        )
        .expect("reverse");
    let forward = engine
        .navigate(
            32.0,
            VersePosition::new(1, 4),
            Direction::Downwards,
            settings,
        )
        .expect("forward");

    assert_eq!(reverse.verse.sura, 1);
    assert_eq!(reverse.verse.number, 1);
    assert_eq!(reverse.cycle_info.cycles_completed(), 2);
    assert_eq!(forward.verse.sura, 1);
    assert_eq!(forward.verse.number, 7);
    assert_eq!(forward.cycle_info.cycles_completed(), 2);
}

#[test]
fn reverse_navigate_ignore_sura_header_changes_destination() {
    let engine = setup_engine();
    let from = VersePosition::new(2, 1);
    let lines = 5.0;

    let with_header = engine
        .reverse_navigate(lines, from, Direction::Downwards, Default::default())
        .expect("with header");
    let without_header = engine
        .reverse_navigate(
            lines,
            from,
            Direction::Downwards,
            NavigationSettings::builder().ignore_sura_header(true),
        )
        .expect("without header");

    assert_eq!(with_header.verse.sura, 1);
    assert_eq!(with_header.verse.number, 7);
    assert_eq!(without_header.verse.sura, 1);
    assert_eq!(without_header.verse.number, 4);
}

#[test]
fn reverse_navigate_fractional_at_quran_start_overflow() {
    let engine = setup_engine();
    let result = engine
        .reverse_navigate(
            0.5,
            VersePosition::start(),
            Direction::Downwards,
            Default::default(),
        )
        .expect("reverse_navigate");
    assert_eq!(result.verse.sura, 1);
    assert_eq!(result.verse.number, 1);
    assert!(result.has_overflow());
    assert!((result.overflow_lines() - 2.3).abs() < 0.15);
}

#[test]
fn helpers_resolve_and_wrong_direction() {
    let engine = setup_engine();
    let loc = engine.find_verse_location(VersePosition::new(1, 1)).expect("location");
    assert_eq!(loc.verse.sura, 1);
    assert_eq!(loc.verse.number, 1);
    assert_eq!(loc.page_number, 1);

    let verse = engine.resolve_position(2, 286).expect("resolve");
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 286);

    assert!(engine.is_wrong_direction(
        VersePosition::new(2, 10),
        VersePosition::new(2, 1),
        Direction::Downwards,
    ));
    assert!(!engine.is_wrong_direction(
        VersePosition::new(2, 1),
        VersePosition::new(2, 10),
        Direction::Downwards,
    ));
}
