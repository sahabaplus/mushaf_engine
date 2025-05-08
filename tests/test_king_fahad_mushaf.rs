#[cfg(test)]
mod test {
    use colored::Colorize;
    use rust_quran_engine::king_fahad_mushaf::KingFahadMushaf;

    use std::path::PathBuf;

    #[test]
    fn load_mushaf() {
        // Use a relative path to the data directory in your project
        let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data_path.push("data");
        data_path.push("king_fahad_mushaf.json");

        let mushaf = KingFahadMushaf::new(data_path.to_str().unwrap());

        // Add some assertions to actually test something
        assert!(mushaf.max_page > 0);
        assert_eq!(mushaf.lines_per_page, 15);
        assert!(!mushaf.pages.is_empty());

        let pages = mushaf.pages;
        for page in pages.iter() {
            // let page = &pages[i];
            let suras_headers: f32 = page
                .verses()
                .iter()
                .map(|v| (
                    if v.number == 1 {
                        if v.sura == 9 {
                            // سورة التوبة, does not have بسملة
                            1f32
                        } else {
                            2f32
                        }
                    } else {
                        0f32
                    }
                ))
                .sum();
            let sum: f32 =
                page
                    .verses()
                    .iter()
                    .fold(0f32, |acc, e| ((acc + e.lines) * 100.0).round() / 100.0) + suras_headers;
            println!("{:#?}", page);
            println!(
                "\t{}:{}\n",
                "Sum".to_string().bold().green(),
                sum.to_string().yellow().bold()
            );
            if page.number() > 2 {
                assert_eq!(sum, 15f32);
            }
        }
    }
}
