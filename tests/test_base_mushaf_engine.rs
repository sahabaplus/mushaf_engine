#[cfg(test)]
mod tests {
    use std::{ path::PathBuf, rc::Rc };
    use colored::Colorize;
    use rust_quran_engine::{
        king_fahad_mushaf::KingFahadMushaf,
        mushaf_engine::{ base_mushaf_engine::BaseMushafEngine, Direction, IMushafEngine },
    };

    fn setup_engine() -> BaseMushafEngine {
        let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data_path.push("data");
        data_path.push("king_fahad_mushaf.json");

        let mushaf = Rc::new(KingFahadMushaf::new(data_path.to_str().unwrap()));
        BaseMushafEngine::new(mushaf)
    }

    #[test]
    fn navigate_15_lines() {
        let engine = setup_engine();

        let v = engine.navigate(15.1f32, 5, 3, Direction::Upwards);
        println!("Found: {}", v);
        // assert!(false);
    }
    #[test]
    fn navigate_upwards() {
        let engine = setup_engine();

        let v = engine.navigate(36.3f32, 2, 282, Direction::Upwards);
        assert_eq!(v.sura, 1);
        assert_eq!(v.number, 6);

        let v = engine.navigate(14.7f32, 114, 1, Direction::Upwards);
        println!("{:?}", engine.mushaf.get_page(604).unwrap());
        println!(
            "Sum: {}",
            (
                engine.mushaf
                    .get_page(604)
                    .unwrap()
                    .verses()
                    .iter()
                    .map(|v| v.lines)
                    .sum::<f32>() + ((3 * 2) as f32)
            )
                .to_string()
                .yellow()
                .bold()
        );
        println!("{}", v);
        assert_eq!(v.sura, 112);
        assert_eq!(v.number, 4);
    }

    #[test]
    fn navigate_zero_lines() {
        let engine = setup_engine();

        // First verse of Al-Fatiha
        let result = engine.navigate(0f32, 1, 1, Direction::Downwards);
        assert_eq!(result.sura, 1);
        assert_eq!(result.number, 1);
    }
}
