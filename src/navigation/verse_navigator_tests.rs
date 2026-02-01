use super::super::*;
use std::{path::PathBuf, rc::Rc};

use colored::Colorize;

use crate::{
    king_fahad_mushaf::{JsonVerse, KingFahadMushaf},
    mushaf::{Mushaf, QuranMetadata, Verse},
    navigation::{Direction, NavigationSettings, VersePosition, VersesNavigator},
};

fn get_mushaf() -> (Rc<Mushaf>, Rc<QuranMetadata>) {
    let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    data_path.push("data");
    data_path.push("king_fahad_mushaf.json");

    let mushaf = Rc::new({
        let path = data_path.to_str().expect("expect `data_path` to be a valid string");
        // Load from provided JSON path
        let file_content = std::fs::read_to_string(path).expect("Failed to read mushaf data file");

        let pages: Vec<Vec<JsonVerse>> =
            serde_json::from_str(&file_content).expect("Failed to parse mushaf JSON data");

        KingFahadMushaf::create_mushaf_from_pages(pages)
    });
    let metadata = Rc::new(QuranMetadata::from_mushaf(&mushaf));

    (mushaf, metadata)
}

#[test]
fn test_calculate_verse_with_headers_and_without_headers() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default())
            .ignore_sura_header(false);
    // Sura (9 - At-Tawbah) does not have a bismillah
    navigator.reset_position(VersePosition::new(9, 1));
    let verse = navigator.current_verse();
    let lines = navigator.calculate_verse_lines(verse);
    assert!((lines - (verse.lines + 1.0)).abs() < f32::EPSILON);

    navigator.reset_position(VersePosition::new(9, 1));
    let mut navigator = navigator.ignore_sura_header(true);
    let verse = navigator.current_verse();
    let lines = navigator.calculate_verse_lines(verse);
    assert_eq!(lines, verse.lines);

    // Loop through all first verses of suras
    navigator = navigator.ignore_sura_header(false);
    for sura in 1..=114 {
        let header_lines = if sura == 9 { 1.0 } else { 2.0 };
        navigator.reset_position(VersePosition::new(sura, 1));
        let verse = navigator.current_verse();
        let lines = navigator.calculate_verse_lines(verse);
        assert!((lines - (verse.lines + header_lines)).abs() < f32::EPSILON);
    }
}

#[test]
fn test_upwards_navigation() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default());
    assert!(navigator.reset_position(VersePosition::new(5, 119)).is_ok());

    let verse = navigator.next_verse();
    let verse = verse.expect("expect `verse` not to be None");
    println!("{verse}");
    assert_eq!(verse.number, 120);
    assert_eq!(verse.sura, 5);

    let mut navigator = navigator.direction(Direction::Upwards);
    let verse = navigator.next_verse();
    let verse = verse.expect("expect `verse` not to be None");
    println!("{verse}");
    assert_eq!(verse.number, 1);
    assert_eq!(verse.sura, 4);
}

#[test]
fn test_downwards_navigation() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default());
    navigator.reset_position(VersePosition::new(5, 119));

    let verse = navigator.next_verse();
    let verse = verse.expect("expect `verse` not to be None");
    println!("{verse}");
    assert_eq!(verse.number, 120);
    assert_eq!(verse.sura, 5);

    let mut navigator = navigator.direction(Direction::Downwards);
    let verse = navigator.next_verse();
    let verse = verse.expect("expect `verse` not to be None");
    println!("{verse}");
    assert_eq!(verse.number, 1);
    assert_eq!(verse.sura, 6);
}

#[test]
fn comprehensive_downwards() {
    let (mushaf, metadata) = get_mushaf();
    let pages = Rc::clone(&mushaf.pages);
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default());

    for page in pages.iter() {
        for verse in page.verses() {
            assert_eq!(*verse, *navigator.current_verse());

            navigator.next_verse_downward();
        }
    }
}
#[test]
fn comprehensive_upwards() {
    let (mushaf, metadata) = get_mushaf();
    let pages = Rc::clone(&mushaf.pages);
    let mut navigator = VersesNavigator::new(
        mushaf,
        metadata.clone(),
        Default::default(),
        Default::default(),
    );

    navigator.reset_position(VersePosition::new(114, 1));
    for i in 1..=114_u8 {
        let sura_number = 114 - i + 1;
        let sura = metadata.get_sura_info(sura_number).expect("Invalid sura number");
        for page_number in sura.start_page..=sura.end_page {
            let page = &pages[(page_number - 1) as usize];
            for verse in page.verses() {
                if verse.sura != sura_number {
                    continue;
                }
                assert_eq!(*verse, *navigator.current_verse());
                navigator.next_verse_upward();
            }
        }
    }
}

#[test]
fn per_sura() {
    let (mushaf, metadata) = get_mushaf();
    let pages = Rc::clone(&mushaf.pages);
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default());
    navigator.reset_position(VersePosition::end());

    let pre = navigator.move_pre_sura(navigator.current_verse().sura);
    let pre = pre.expect("expect `pre` not to be Err");
    assert_eq!(pre.sura, 113);

    for i in 1..=113 {
        let sura = 115 - i;
        let pre = navigator.move_pre_sura(sura);
        let pre = pre.expect("expect `pre` not to be Err");
        assert_eq!(pre.sura, sura - 1);
    }
}

// test reset iterations
#[test]
fn test_move_by_index_forward_same_page() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default());

    // Start at first verse of first page
    navigator.reset_position(VersePosition::new(1, 1));

    // Move forward by 2 verses within the same page
    let verse = navigator.forward_index(2);
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 3); // Should be 3rd verse of Al-Fatiha

    // Move forward by 1 more verse
    let verse = navigator.forward_index(1);
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");

    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 4);
}

#[test]
fn test_move_by_index_backward_same_page() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator = VersesNavigator::new(
        mushaf.clone(),
        metadata,
        Default::default(),
        Default::default(),
    );
    // Start at 4th verse of first page
    navigator.reset_position(VersePosition::new(1, 4));

    // Move backward by 2 verses within the same page
    let verse = navigator.backward_index(2);
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 2); // Should be 2nd verse of Al-Fatiha

    // Move backward by 1 more verse
    let verse = navigator.backward_index(1);
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 1);
}

#[test]
fn test_move_by_index_forward_cross_page() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator = VersesNavigator::new(
        mushaf.clone(),
        metadata.clone(),
        Default::default(),
        Default::default(),
    );

    // Start at first verse of first page
    navigator.reset_position(VersePosition::new(1, 1));

    // Get the number of verses in the first page
    let first_page_verses = mushaf.pages[0].verses();
    let first_page_verse_count = first_page_verses.len();

    let verse = navigator.forward_index(first_page_verse_count + 2);
    assert!(verse.is_some());
}

#[test]
fn test_move_by_index_backward_cross_page() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator = VersesNavigator::new(
        mushaf.clone(),
        metadata.clone(),
        Default::default(),
        Default::default(),
    );

    // Start at a verse in the second page
    navigator.reset_position(VersePosition::new(2, 1));

    // Get the number of verses in the first page
    let first_page_verses = mushaf.pages[0].verses();
    let first_page_verse_count = first_page_verses.len();

    // Move backward by more verses than available in current page
    let verse = navigator.backward_index(first_page_verse_count + 2);
    assert!(verse.is_none());
}

#[test]
fn test_move_by_index_zero_index() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default());

    navigator.reset_position(VersePosition::new(1, 1));

    // Move by 0 index should return None
    let verse = navigator.forward_index(0).copied();
    assert!(verse.is_some());
    assert_eq!(
        VersePosition::new(1, 1),
        verse.expect("expect `verse` not to be None")
    );

    let verse = navigator.backward_index(0).copied();
    assert!(verse.is_some());
    assert_eq!(
        VersePosition::new(1, 1),
        verse.expect("expect `verse` not to be None")
    );
}

#[test]
fn test_move_by_index_large_index() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default());

    navigator.reset_position(VersePosition::new(1, 1));

    // Try to move by a very large index that exceeds all available verses
    let verse = navigator.forward_index(100000);
    assert!(verse.is_none());

    navigator.reset_position(VersePosition::new(1, 1));
    let verse = navigator.backward_index(100000);
    assert!(verse.is_none());
}

#[test]
fn test_move_by_index_at_page_boundaries() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator = VersesNavigator::new(
        mushaf.clone(),
        metadata.clone(),
        Default::default(),
        Default::default(),
    );

    // Test at the last verse of a page
    let first_page_verses = mushaf.pages[0].verses();
    let last_verse_idx = first_page_verses.len() - 1;
    let last_verse = &first_page_verses[last_verse_idx];

    navigator.reset_position(VersePosition::new(last_verse.sura, last_verse.number));

    // Move forward by 1 should go to next page
    let verse = navigator.forward_index(1);
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    // Should be in the second page
    assert!(mushaf.pages[1].verses().contains(verse));

    // Test at the first verse of a page
    navigator.reset_position(VersePosition::new(2, 1));

    // Move backward by 1 should go to previous page
    let verse = navigator.backward_index(1);
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    // Should be in the first page
    assert!(mushaf.pages[0].verses().contains(verse));
}

#[test]
fn test_move_by_index_consistency_with_sequential_navigation() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator = VersesNavigator::new(
        mushaf.clone(),
        metadata.clone(),
        Default::default(),
        Default::default(),
    );

    navigator.reset_position(VersePosition::new(1, 1));

    // Move forward by 3 using index navigation
    let verse_by_index = navigator.forward_index(3).copied();
    assert!(verse_by_index.is_some());
    let verse_by_index = verse_by_index.expect("expect `verse_by_index` not to be None");

    // Reset and move forward 3 times using sequential navigation
    navigator.reset_position(VersePosition::new(1, 1));
    let mut verse_by_sequential = navigator.current_verse();
    for _ in 0..3 {
        verse_by_sequential =
            navigator.next_verse().expect("expect `verse_by_sequential` not to be None");
    }

    // Results should be the same
    assert_eq!(verse_by_index.sura, verse_by_sequential.sura);
    assert_eq!(verse_by_index.number, verse_by_sequential.number);
}

#[test]
fn test_move_by_index_with_different_directions() {
    let (mushaf, metadata) = get_mushaf();

    // Test with Upwards direction
    let mut navigator_up = VersesNavigator::new(
        Rc::clone(&mushaf),
        Rc::clone(&metadata),
        Default::default(),
        Direction::Upwards,
    );

    navigator_up.reset_position(VersePosition::new(2, 5));
    let verse_up = navigator_up.forward_index(3);
    assert!(verse_up.is_some());

    // Test with Downwards direction
    let mut navigator_down =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards);

    navigator_down.reset_position(VersePosition::new(2, 5));
    let verse_down = navigator_down.forward_index(3);
    assert!(verse_down.is_some());

    // Both should return valid verses (though potentially different due to direction)
    assert!(verse_up.expect("expect `verse_up` not to be None").sura > 0);
    assert!(verse_down.expect("expect `verse_down` not to be None").sura > 0);
}

#[test]
fn test_move_by_index_invalid_page_index() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default());

    // Set an invalid page index (beyond available pages)
    navigator.current_page_idx = 1000;

    // Any movement should return None
    let verse = navigator.forward_index(1);
    assert!(verse.is_none());

    let verse = navigator.backward_index(1);
    assert!(verse.is_none());
}

#[test]
fn test_bounds_validation() {
    // Test that upper_bound must be less than lower_bound
    let settings = NavigationSettings::builder()
        .upper_bound(VersePosition::new(2, 1))
        .lower_bound(VersePosition::new(2, 286));
    assert_eq!(settings.bounds.upper_bound, VersePosition::new(2, 1));
    assert_eq!(settings.bounds.lower_bound, VersePosition::new(2, 286));
}

#[test]
fn test_bounds_remain_consistent_with_direction_changes() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards);

    let upper_bound = navigator.settings.bounds.upper_bound;
    let lower_bound = navigator.settings.bounds.lower_bound;

    navigator = navigator.direction(Direction::Upwards);
    // Bounds should remain the same regardless of direction
    assert_eq!(navigator.settings.bounds.upper_bound, upper_bound);
    assert_eq!(navigator.settings.bounds.lower_bound, lower_bound);
}

#[test]
fn test_reset_position() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default());
    navigator.reset_position(VersePosition::new(2, 1));
    assert_eq!(navigator.current_verse().sura, 2);
    assert_eq!(navigator.current_verse().number, 1);

    navigator.reset_position(VersePosition::new(2, 286));
    assert_eq!(navigator.current_verse().sura, 2);
    assert_eq!(navigator.current_verse().number, 286);

    navigator.reset_position(VersePosition::end());
    assert_eq!(navigator.current_verse().sura, 114);
    assert_eq!(navigator.current_verse().number, 6);
}

#[test]
fn test_reset_position_invalid_verse() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default());

    assert!(navigator.reset_position(VersePosition::new(2, 287)).is_err());
    assert!(navigator.reset_position(VersePosition::new(2, 0)).is_err());
    assert!(navigator.reset_position(VersePosition::new(114, 7)).is_err());
    assert!(navigator.reset_position(VersePosition::new(114, 0)).is_err());
    assert!(navigator.reset_position(VersePosition::new(112, 90)).is_err());
}

#[test]
fn test_multiple_iterations() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default())
            .upper_bound(VersePosition::new(2, 1))
            .lower_bound(VersePosition::new(2, 286))
            .iteration_limit(1);
    navigator.reset_position(VersePosition::new(2, 1));

    let sura_2_lines = {
        let (mushaf, metadata) = get_mushaf();
        let mut nav =
            VersesNavigator::new(mushaf, metadata, Default::default(), Default::default());
        nav.reset_position(VersePosition::new(2, 1));
        let mut lines = 0.0;
        println!("===== Start of `sura_2_lines` =====");
        loop {
            let verse = nav.current_verse();
            if verse.sura != 2 {
                break;
            }
            if verse.number == 1 || verse.number == 286 {
                println!("{verse}");
            } else {
                print!(".");
            }
            lines += verse.lines;

            nav.next_verse();
        }
        println!("===== End of `sura_2_lines` =====");
        lines
    };

    let mut lines_count = 0.0;
    println!("===== Start of `lines_count` =====");
    loop {
        let verse = navigator.current_verse();
        if verse.number == 1 || verse.number == 286 {
            println!("{verse}");
        } else {
            print!(".");
        }
        if verse.sura != 2 {
            unreachable!();
        }
        lines_count += verse.lines;

        if navigator.next_verse().is_none() {
            break;
        }
    }
    println!("===== End of `lines_count` =====");
    assert!((sura_2_lines * 2.0 - lines_count).abs() < f32::EPSILON);
}

#[test]
fn test_excluding_mode_basic_downwards() {
    let (mushaf, metadata) = get_mushaf();
    // Exclude verses strictly between 2 and 4 of Sura 1 (exclusive)
    // So we should navigate: (1,1), (1,2), skip (1,3), (1,4), (1,5), (1,6), (1,7)
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default())
            .upper_bound(VersePosition::new(1, 4)) // upper > lower = excluding mode
            .lower_bound(VersePosition::new(1, 2))
            .iteration_limit(0);

    navigator
        .reset_position(VersePosition::new(1, 1))
        .expect("Should reset to (1,1)");

    // First verse should be (1,1)
    let verse = navigator.current_verse();
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 1);

    // Next should be (1,2) - bound itself is NOT excluded
    let verse = navigator.next_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 2);

    // Next should skip to (1,4) - skipping (1,3) which is strictly between bounds
    let verse = navigator.next_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 4);

    // Continue to (1,5)
    let verse = navigator.next_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 5);

    // Continue to (1,6)
    let verse = navigator.next_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 6);

    // Continue to (1,7)
    let verse = navigator.next_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 7);

    // Should continue to next sura (2,1)
    let verse = navigator.next_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 1);
}

#[test]
fn test_is_out_of_bounds_inclusive_mode() {
    let (mushaf, metadata) = get_mushaf();
    // Normal inclusive mode: upper_bound < lower_bound
    // Navigate from (2, 1) to (2, 286) - all of Sura 2
    let navigator = VersesNavigator::new(mushaf, metadata, Default::default(), Default::default())
        .upper_bound(VersePosition::new(2, 1))
        .lower_bound(VersePosition::new(2, 286));

    // Verses within the inclusive range should NOT be out of bounds
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 1)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 50)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 286)));

    // Verses outside the inclusive range should be out of bounds
    assert!(navigator.is_out_of_bounds(VersePosition::new(1, 7))); // Before upper_bound
    assert!(navigator.is_out_of_bounds(VersePosition::new(3, 1))); // After lower_bound
}

#[test]
fn test_is_out_of_bounds_excluding_mode() {
    let (mushaf, metadata) = get_mushaf();
    // Excluding mode: upper_bound > lower_bound
    // Navigate everywhere EXCEPT verses strictly between 2 and 4 of Sura 1 (exclusive)
    let navigator = VersesNavigator::new(mushaf, metadata, Default::default(), Default::default())
        .upper_bound(VersePosition::new(1, 4))
        .lower_bound(VersePosition::new(1, 2));

    // Only verse 3 is in the excluded range (exclusive bounds)
    assert!(navigator.is_out_of_bounds(VersePosition::new(1, 3)));

    // Verses 2 and 4 are the bounds themselves, so NOT excluded
    assert!(!navigator.is_out_of_bounds(VersePosition::new(1, 2)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(1, 4)));

    // Verses outside excluded range should NOT be out of bounds
    assert!(!navigator.is_out_of_bounds(VersePosition::new(1, 1)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(1, 5)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(1, 6)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(1, 7)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 1)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(114, 6)));
}

#[test]
fn test_is_out_of_bounds_excluding_mode_large_range() {
    let (mushaf, metadata) = get_mushaf();
    // Exclude a large range: verses strictly between 10 and 100 of Sura 2 (exclusive)
    let navigator = VersesNavigator::new(mushaf, metadata, Default::default(), Default::default())
        .upper_bound(VersePosition::new(2, 100))
        .lower_bound(VersePosition::new(2, 10));

    // Verses strictly between 10 and 100 are excluded
    assert!(navigator.is_out_of_bounds(VersePosition::new(2, 11)));
    assert!(navigator.is_out_of_bounds(VersePosition::new(2, 50)));
    assert!(navigator.is_out_of_bounds(VersePosition::new(2, 99)));

    // Verses 10 and 100 are the bounds themselves, so NOT excluded
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 10)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 100)));

    // Verses outside excluded range should NOT be out of bounds
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 9)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 101)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(1, 1)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(3, 1)));
}

#[test]
fn test_is_out_of_bounds_excluding_mode_cross_sura() {
    let (mushaf, metadata) = get_mushaf();
    // Exclude range across suras: strictly between (1, 5) and (2, 5) (exclusive)
    let navigator = VersesNavigator::new(mushaf, metadata, Default::default(), Default::default())
        .upper_bound(VersePosition::new(2, 5))
        .lower_bound(VersePosition::new(1, 5));

    // Verses strictly between (1,5) and (2,5) are excluded
    assert!(navigator.is_out_of_bounds(VersePosition::new(1, 6)));
    assert!(navigator.is_out_of_bounds(VersePosition::new(1, 7)));
    assert!(navigator.is_out_of_bounds(VersePosition::new(2, 1)));
    assert!(navigator.is_out_of_bounds(VersePosition::new(2, 2)));
    assert!(navigator.is_out_of_bounds(VersePosition::new(2, 3)));
    assert!(navigator.is_out_of_bounds(VersePosition::new(2, 4)));

    // The bounds themselves (1,5) and (2,5) are NOT excluded
    assert!(!navigator.is_out_of_bounds(VersePosition::new(1, 5)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 5)));

    // Verses outside excluded range should NOT be out of bounds
    assert!(!navigator.is_out_of_bounds(VersePosition::new(1, 4)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 6)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(3, 1)));
}

#[test]
fn test_is_out_of_bounds_invalid_verse() {
    let (mushaf, metadata) = get_mushaf();
    let navigator = VersesNavigator::new(mushaf, metadata, Default::default(), Default::default());

    // Invalid verses should be out of bounds
    assert!(navigator.is_out_of_bounds(VersePosition::new(0, 1)));
    assert!(navigator.is_out_of_bounds(VersePosition::new(115, 1)));
    assert!(navigator.is_out_of_bounds(VersePosition::new(1, 0)));
    assert!(navigator.is_out_of_bounds(VersePosition::new(1, 100))); // Al-Fatiha only has 7 verses
}

#[test]
fn test_excluding_mode_reset_position() {
    let (mushaf, metadata) = get_mushaf();
    // Exclude verses strictly between 2 and 4 of Sura 1 (exclusive)
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default())
            .upper_bound(VersePosition::new(1, 4))
            .lower_bound(VersePosition::new(1, 2));

    // Should be able to reset to verses outside excluded range
    assert!(navigator.reset_position(VersePosition::new(1, 1)).is_ok());
    assert!(navigator.reset_position(VersePosition::new(1, 5)).is_ok());
    assert!(navigator.reset_position(VersePosition::new(2, 1)).is_ok());

    // Bounds themselves (2 and 4) are NOT excluded, so they should work
    assert!(navigator.reset_position(VersePosition::new(1, 2)).is_ok());
    assert!(navigator.reset_position(VersePosition::new(1, 4)).is_ok());

    // Should NOT be able to reset to verse 3 (strictly between bounds)
    assert!(navigator.reset_position(VersePosition::new(1, 3)).is_err());
}

#[test]
fn test_excluding_mode_cross_sura() {
    let (mushaf, metadata) = get_mushaf();
    // Exclude verses strictly between 1 and 5 of Sura 2 (exclusive)
    // This tests skipping across sura boundaries
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default())
            .upper_bound(VersePosition::new(2, 5))
            .lower_bound(VersePosition::new(2, 1))
            .iteration_limit(0);

    // Start at end of Sura 1
    navigator
        .reset_position(VersePosition::new(1, 7))
        .expect("Should reset to (1,7)");

    // Next should be (2,1) - the lower bound itself is NOT excluded
    let verse = navigator.next_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 1);

    // Next should skip to (2,5) - skipping (2,2), (2,3), (2,4)
    let verse = navigator.next_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 5);

    // Next should be (2,6)
    let verse = navigator.next_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 6);
}

#[test]
fn test_excluding_mode_get_bounds() {
    let (mushaf, metadata) = get_mushaf();
    // Excluding mode: exclude verses strictly between (2,286) and (78,1)
    let mut navigator = VersesNavigator::new(
        Rc::clone(&mushaf),
        Rc::clone(&metadata),
        Default::default(),
        Direction::Upwards,
    )
    .upper_bound(VersePosition::new(78, 1)) // An-Naba' starts at verse 1
    .lower_bound(VersePosition::new(2, 286)); // Al-Baqarah ends at verse 286

    // In excluding mode with upwards direction:
    // start_bound = start of sura containing lower_bound = (2, 1)
    let start_bound = navigator.get_start_bound();
    assert_eq!(start_bound, VersePosition::new(2, 1));
    navigator.reset_position(start_bound).expect("Should reset to (2,1)");

    loop {
        let verse = navigator.next_verse();
        if verse.is_none() {
            break;
        }
        let verse = verse.expect("expect `verse` not to be None");
        println!("{verse}");
    }

    // end_bound = end of sura containing upper_bound
    let end_bound = navigator.get_end_bound();
    let sura_78_info = metadata.get_sura_info(78).expect("Sura 78 should exist");
    assert_eq!(end_bound, VersePosition::new(78, sura_78_info.total_verses));
}

#[test]
fn test_excluding_mode_large_range() {
    let (mushaf, metadata) = get_mushaf();
    // Exclude verses strictly between 10 and 100 of Sura 2 (exclusive)
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Default::default())
            .upper_bound(VersePosition::new(2, 100))
            .lower_bound(VersePosition::new(2, 10))
            .iteration_limit(0);

    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 10)));

    navigator
        .reset_position(VersePosition::new(2, 10))
        .expect("Should reset to (2,10)");

    // Next should skip to (2,100) - skipping (2,11)-(2,99)
    let verse = navigator.next_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect `verse` not to be None");
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 100); // Bound itself is NOT excluded

    // Verify excluded verses are out of bounds
    assert!(navigator.is_out_of_bounds(VersePosition::new(2, 11))); // Strictly between bounds
    assert!(navigator.is_out_of_bounds(VersePosition::new(2, 50)));
    assert!(navigator.is_out_of_bounds(VersePosition::new(2, 99)));

    // Bounds themselves and outside are NOT excluded
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 9)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 10)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 100)));
    assert!(!navigator.is_out_of_bounds(VersePosition::new(2, 101)));
}

#[test]
fn test_get_start_and_end_bounds() {
    let (mushaf, metadata) = get_mushaf();

    // Test 1: Downwards direction - start_bound should be upper_bound
    let navigator = VersesNavigator::new(
        Rc::clone(&mushaf),
        Rc::clone(&metadata),
        Default::default(),
        Direction::Downwards,
    )
    .upper_bound(VersePosition::new(2, 1))
    .lower_bound(VersePosition::new(2, 286));

    assert_eq!(navigator.get_start_bound(), VersePosition::new(2, 1));
    assert_eq!(navigator.get_end_bound(), VersePosition::new(2, 286));

    // Test 2: Upwards direction with full sura in bounds
    let navigator = VersesNavigator::new(
        Rc::clone(&mushaf),
        Rc::clone(&metadata),
        Default::default(),
        Direction::Upwards,
    )
    .upper_bound(VersePosition::new(2, 1))
    .lower_bound(VersePosition::new(2, 286));

    // Should be start of sura 2
    assert_eq!(navigator.get_start_bound(), VersePosition::new(2, 1));
    // Should be end of sura 2
    assert_eq!(navigator.get_end_bound(), VersePosition::new(2, 286));

    // Test 3: Upwards with start of sura out of bounds
    let navigator = VersesNavigator::new(
        Rc::clone(&mushaf),
        Rc::clone(&metadata),
        Default::default(),
        Direction::Upwards,
    )
    .upper_bound(VersePosition::new(2, 10))
    .lower_bound(VersePosition::new(2, 50));

    // Start of sura 2 (verse 1) is out of bounds (< 10), should return upper_bound
    assert_eq!(navigator.get_start_bound(), VersePosition::new(2, 10));
    // End of sura 2 (verse 286) is out of bounds (> 50), should return lower_bound
    assert_eq!(navigator.get_end_bound(), VersePosition::new(2, 50));

    // Test 4: Upwards with bounds spanning multiple suras
    let navigator = VersesNavigator::new(
        Rc::clone(&mushaf),
        Rc::clone(&metadata),
        Default::default(),
        Direction::Upwards,
    )
    .upper_bound(VersePosition::new(1, 1))
    .lower_bound(VersePosition::new(3, 100));

    // Should be start of sura 3
    assert_eq!(navigator.get_start_bound(), VersePosition::new(3, 1));
    // Should be end of sura 1 (verse 7)
    assert_eq!(navigator.get_end_bound(), VersePosition::new(1, 7));

    // Test 5: Bounds consistency when switching directions
    let mut navigator = VersesNavigator::new(
        Rc::clone(&mushaf),
        Rc::clone(&metadata),
        Default::default(),
        Direction::Downwards,
    )
    .upper_bound(VersePosition::new(5, 1))
    .lower_bound(VersePosition::new(5, 120));

    // Downwards: start=upper, end=lower
    assert_eq!(navigator.get_start_bound(), VersePosition::new(5, 1));
    assert_eq!(navigator.get_end_bound(), VersePosition::new(5, 120));

    // Switch to upwards
    navigator = navigator.direction(Direction::Upwards);

    // Upwards: start=start of sura, end=end of sura
    assert_eq!(navigator.get_start_bound(), VersePosition::new(5, 1));
    assert_eq!(navigator.get_end_bound(), VersePosition::new(5, 120));

    // Test 6: Partial sura coverage with upwards direction
    let navigator = VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Upwards)
        .upper_bound(VersePosition::new(2, 50))
        .lower_bound(VersePosition::new(2, 100));

    // Start of sura 2 (verse 1) is out of bounds (< 50), should return upper_bound
    assert_eq!(navigator.get_start_bound(), VersePosition::new(2, 50));
    // End of sura 2 (verse 286) is out of bounds (> 100), should return lower_bound
    assert_eq!(navigator.get_end_bound(), VersePosition::new(2, 100));
}

// ==================== previous_verse tests ====================

#[test]
fn test_previous_verse_basic_downward() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards);

    // Start at verse 5 of Al-Fatiha
    navigator
        .reset_position(VersePosition::new(1, 5))
        .expect("Should reset to (1,5)");
    assert_eq!(navigator.current_verse().number, 5);

    // Go back to verse 4
    let verse = navigator.previous_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 4);

    // Go back to verse 3
    let verse = navigator.previous_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 3);
}

#[test]
fn test_previous_verse_cross_sura_downward() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards);

    // Start at verse 1 of Sura 2 (Al-Baqarah)
    navigator
        .reset_position(VersePosition::new(2, 1))
        .expect("Should reset to (2,1)");

    // Go back - should cross to Sura 1 verse 7
    let verse = navigator.previous_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 7);
}

#[test]
fn test_previous_verse_at_start_of_quran_downward_no_wrap() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards);

    // Start at verse 1 of Sura 1 (start of Quran)
    navigator
        .reset_position(VersePosition::new(1, 1))
        .expect("Should reset to (1,1)");

    // With default settings (upper_bound = start), previous returns None at start
    let verse = navigator.previous_verse();
    assert!(
        verse.is_none(),
        "At start of Quran with default bounds, previous should return None"
    );
}

#[test]
fn test_previous_verse_at_start_with_iteration_cycles_to_end() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards)
            .iteration_limit(1);

    // Start at verse 1 of Sura 1 (start of Quran)
    navigator
        .reset_position(VersePosition::new(1, 1))
        .expect("Should reset to (1,1)");

    // With iteration_limit = 1, at start bound should cycle to end bound
    let verse = navigator.previous_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 114);
    assert_eq!(verse.number, 6); // End of Quran
}

#[test]
fn test_previous_verse_basic_upward() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Upwards);

    // In upward direction, start at verse 3 of Sura 2
    navigator
        .reset_position(VersePosition::new(2, 3))
        .expect("Should reset to (2,3)");

    // Previous in upward direction goes backward within sura first
    let verse = navigator.previous_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 2);
}

#[test]
fn test_previous_verse_cross_sura_upward() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Upwards);

    // Start at first verse of Sura 2
    navigator
        .reset_position(VersePosition::new(2, 1))
        .expect("Should reset to (2,1)");

    // Previous in upward direction should go to end of Sura 3
    let verse = navigator.previous_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 3);
    // Sura 3 (Ali 'Imran) has 200 verses
    assert_eq!(verse.number, 200);
}

#[test]
fn test_previous_verse_with_bounds_and_iteration() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards)
            .upper_bound(VersePosition::new(1, 1))
            .lower_bound(VersePosition::new(1, 7))
            .iteration_limit(2);

    // Start at verse 3
    navigator
        .reset_position(VersePosition::new(1, 3))
        .expect("Should reset to (1,3)");

    // Go back to verse 2
    let verse = navigator.previous_verse();
    assert!(verse.is_some());
    assert_eq!(verse.expect("expect verse").number, 2);

    // Go back to verse 1 (start bound)
    let verse = navigator.previous_verse();
    assert!(verse.is_some());
    assert_eq!(verse.expect("expect verse").number, 1);

    // At start bound with iterations remaining, should cycle to end bound (verse 7)
    let verse = navigator.previous_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 7);
}

#[test]
fn test_previous_verse_stops_at_bound_without_iterations() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards)
            .upper_bound(VersePosition::new(1, 1))
            .lower_bound(VersePosition::new(1, 7))
            .iteration_limit(0); // No cycling

    // Start at verse 1 (start bound)
    navigator
        .reset_position(VersePosition::new(1, 1))
        .expect("Should reset to (1,1)");

    // At start bound with no iterations, should return None
    let verse = navigator.previous_verse();
    assert!(verse.is_none());
}

#[test]
fn test_previous_verse_excluding_mode() {
    let (mushaf, metadata) = get_mushaf();
    // Exclude verses strictly between (2,10) and (2,20) - so verses 11-19 are excluded
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards)
            .upper_bound(VersePosition::new(2, 20)) // upper > lower = excluding mode
            .lower_bound(VersePosition::new(2, 10))
            .iteration_limit(0);

    // Start at verse 20 (upper bound, not excluded)
    navigator
        .reset_position(VersePosition::new(2, 20))
        .expect("Should reset to (2,20)");

    // Previous should skip to verse 10 (lower bound, not excluded)
    // because verses 11-19 are excluded
    let verse = navigator.previous_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 10);

    // Next previous should be verse 9
    let verse = navigator.previous_verse();
    assert!(verse.is_some());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 2);
    assert_eq!(verse.number, 9);
}

#[test]
fn test_next_then_previous_returns_to_original() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards);

    // Start at a middle verse
    navigator
        .reset_position(VersePosition::new(2, 100))
        .expect("Should reset to (2,100)");

    let original_verse = *navigator.current_verse();

    // Go forward
    navigator.next_verse();
    let next_verse = *navigator.current_verse();
    assert_eq!(next_verse.number, 101);

    // Go back
    navigator.previous_verse();
    let back_verse = *navigator.current_verse();

    // Should be back at original
    assert_eq!(back_verse.sura, original_verse.sura);
    assert_eq!(back_verse.number, original_verse.number);
}

#[test]
fn test_previous_then_next_returns_to_original() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards);

    // Start at a middle verse
    navigator
        .reset_position(VersePosition::new(2, 100))
        .expect("Should reset to (2,100)");

    let original_verse = *navigator.current_verse();

    // Go backward
    navigator.previous_verse();
    let prev_verse = *navigator.current_verse();
    assert_eq!(prev_verse.number, 99);

    // Go forward
    navigator.next_verse();
    let forward_verse = *navigator.current_verse();

    // Should be back at original
    assert_eq!(forward_verse.sura, original_verse.sura);
    assert_eq!(forward_verse.number, original_verse.number);
}

#[test]
fn test_comprehensive_previous_downwards() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards);

    // Start at end of Quran
    navigator.reset_position(VersePosition::end()).expect("Should reset to end");

    // Navigate backward through several verses
    let mut prev_verse = *navigator.current_verse();
    for _ in 0..20 {
        let verse = navigator.previous_verse();
        assert!(verse.is_some(), "Should be able to go backward");
        let verse = verse.expect("expect verse");

        // Verify we're actually going backward
        let prev_pos = VersePosition::new(prev_verse.sura, prev_verse.number);
        let curr_pos = VersePosition::new(verse.sura, verse.number);
        assert!(
            curr_pos < prev_pos,
            "Current {:?} should be less than previous {:?}",
            curr_pos,
            prev_pos
        );

        prev_verse = *verse;
    }
}

#[test]
fn test_move_next_sura() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Upwards);

    // Start at first verse of Sura 2
    navigator
        .reset_position(VersePosition::new(2, 1))
        .expect("Should reset to (2,1)");

    // move_next_sura should go to end of Sura 3
    let verse = navigator.move_next_sura(2);
    assert!(verse.is_ok());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 3);
    assert_eq!(verse.number, 200); // Sura 3 has 200 verses

    // From sura 114, should wrap to sura 1
    let verse = navigator.move_next_sura(114);
    assert!(verse.is_ok());
    let verse = verse.expect("expect verse");
    assert_eq!(verse.sura, 1);
    assert_eq!(verse.number, 7); // Sura 1 has 7 verses
}

#[test]
fn test_try_retreat_within_current_sura() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator =
        VersesNavigator::new(mushaf, metadata, Default::default(), Direction::Downwards);

    // Start at verse 5 of Sura 2
    navigator
        .reset_position(VersePosition::new(2, 5))
        .expect("Should reset to (2,5)");

    // Should succeed - moving back within same sura
    let result = navigator.try_retreat_within_current_sura();
    assert!(result);
    assert_eq!(navigator.current_verse().number, 4);
    assert_eq!(navigator.current_verse().sura, 2);

    // Reset to verse 1 of Sura 2
    navigator
        .reset_position(VersePosition::new(2, 1))
        .expect("Should reset to (2,1)");

    // Should fail - can't retreat within sura from verse 1
    let result = navigator.try_retreat_within_current_sura();
    assert!(!result);
    // Position should be unchanged
    assert_eq!(navigator.current_verse().number, 1);
    assert_eq!(navigator.current_verse().sura, 2);
}

#[test]
fn test_full_quran_navigation_cycle() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator = VersesNavigator::new(
        mushaf,
        metadata,
        NavigationSettings::builder()
            .upper_bound(VersePosition::new(14, 52))
            .lower_bound(VersePosition::new(7, 1)),
        Direction::Downwards,
    );

    navigator.reset_position(VersePosition::start());
    let stdout = std::io::stdout();
    use std::io::Write;
    let mut writer = stdout.lock();

    let mut c = 0;

    while let Some(v) = navigator.next_verse() {}

    assert_eq!(&VersePosition::end(), navigator.current_verse());

    while let Some(v) = navigator.previous_verse() {}
    assert_eq!(&VersePosition::start(), navigator.current_verse());
}
#[test]

fn manual_tests() {
    let (mushaf, metadata) = get_mushaf();
    let mut navigator = VersesNavigator::new(
        mushaf,
        metadata,
        NavigationSettings::builder()
            .upper_bound(VersePosition::new(15, 9))
            .lower_bound(VersePosition::new(6, 165)),
        Direction::Downwards,
    );

    let mut sura = 0;
    use crate::mushaf::Verse;
    let mut pre = None;
    while let Some(v) = navigator.next_verse() {
        if sura != v.sura {
            sura = v.sura;
            if let Some(pre) = pre {
                println!("{}", pre);
            }
            println!("{}", v);
        }
        if sura == 15 {
            println!("==> {}", v);
        }
        pre = Some(*v);
    }

    navigator.reset_position(VersePosition::start());
}
