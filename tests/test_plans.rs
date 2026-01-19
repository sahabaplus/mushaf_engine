#[cfg(test)]
mod test {
    #[allow(unused)]
    const QURAN_SURAS: [&str; 115] = [
        "NONE",
        "الْفَاتِحَةُ",
        "الْبَقَرَةُ",
        "آلِ عِمْرَانَ",
        "النِّسَاءُ",
        "الْمَائِدَةُ",
        "الْأَنْعَامُ",
        "الْأَعْرَافُ",
        "الْأَنْفَالُ",
        "التَّوْبَةُ",
        "يُونُسُ",
        "هُودٌ",
        "يُوسُفُ",
        "الرَّعْدُ",
        "إِبْرَاهِيمُ",
        "الْحِجْرُ",
        "النَّحْلُ",
        "الْإِسْرَاءُ",
        "الْكَهْفُ",
        "مَرْيَمُ",
        "طٰهٰ",
        "الْأَنْبِيَاءُ",
        "الْحَجُّ",
        "الْمُؤْمِنُونَ",
        "النُّورُ",
        "الْفُرْقَانُ",
        "الشُّعَرَاءُ",
        "النَّمْلُ",
        "الْقَصَصُ",
        "الْعَنْكَبُوتُ",
        "الرُّومُ",
        "لُقْمَانُ",
        "السَّجْدَةُ",
        "الْأَحْزَابُ",
        "سَبَأٌ",
        "فَاطِرُ",
        "يٰسۤ",
        "الصَّافَّاتُ",
        "صۤ",
        "الزُّمَرُ",
        "غَافِرٌ",
        "فُصِّلَتْ",
        "الشُّورَىٰ",
        "الزُّخْرُفُ",
        "الدُّخَانُ",
        "الْجَاثِيَةُ",
        "الْأَحْقَافُ",
        "مُحَمَّدٌ",
        "الْفَتْحُ",
        "الْحُجُرَاتُ",
        "قۤ",
        "الذَّارِيَاتُ",
        "الطُّورُ",
        "النَّجْمُ",
        "الْقَمَرُ",
        "الرَّحْمٰنُ",
        "الْوَاقِعَةُ",
        "الْحَدِيدُ",
        "الْمُجَادَلَةُ",
        "الْحَشْرُ",
        "الْمُمْتَحَنَةُ",
        "الصَّفُّ",
        "الْجُمُعَةُ",
        "الْمُنَافِقُونَ",
        "التَّغَابُنُ",
        "الطَّلَاقُ",
        "التَّحْرِيمُ",
        "الْمُلْكُ",
        "الْقَلَمُ",
        "الْحَاقَّةُ",
        "الْمَعَارِجُ",
        "نُوحٌ",
        "الْجِنُّ",
        "الْمُزَّمِّلُ",
        "الْمُدَّثِّرُ",
        "الْقِيَامَةُ",
        "الْإِنْسَانُ",
        "الْمُرْسَلَاتُ",
        "النَّبَأُ",
        "النَّازِعَاتُ",
        "عَبَسَ",
        "التَّكْوِيرُ",
        "الْانْفِطَارُ",
        "الْمُطَفِّفِينَ",
        "الْانْشِقَاقُ",
        "الْبُرُوجُ",
        "الطَّارِقُ",
        "الْأَعْلَىٰ",
        "الْغَاشِيَةُ",
        "الْفَجْرُ",
        "الْبَلَدُ",
        "الشَّمْسُ",
        "اللَّيْلُ",
        "الضُّحَىٰ",
        "الشَّرْحُ",
        "التِّينُ",
        "الْعَلَقُ",
        "الْقَدْرُ",
        "الْبَيِّنَةُ",
        "الزَّلْزَلَةُ",
        "الْعَادِيَاتُ",
        "الْقَارِعَةُ",
        "التَّكَاثُرُ",
        "الْعَصْرُ",
        "الْهُمَزَةُ",
        "الْفِيلُ",
        "قُرَيْشٌ",
        "الْمَاعُونُ",
        "الْكَوْثَرُ",
        "الْكَافِرُونَ",
        "النَّصْرُ",
        "الْمَسَدُ",
        "الْإِخْلَاصُ",
        "الْفَلَقُ",
        "النَّاسُ",
    ];
    use rust_quran_engine::{
        king_fahad_mushaf::{JsonVerse, KingFahadMushaf},
        mushaf_engine::{IMushafEngine, base_mushaf_engine::BaseMushafEngine},
        navigation::{Direction, VersePosition},
    };
    use std::{path::PathBuf, rc::Rc};

    fn setup_engine() -> BaseMushafEngine {
        let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data_path.push("data");
        data_path.push("king_fahad_mushaf.json");

        let mushaf = Rc::new({
            let path = data_path
                .to_str()
                .expect("expect `data_path` to be a valid string");
            // Load from provided JSON path
            let file_content =
                std::fs::read_to_string(path).expect("Failed to read mushaf data file");

            let pages: Vec<Vec<JsonVerse>> =
                serde_json::from_str(&file_content).expect("Failed to parse mushaf JSON data");

            KingFahadMushaf::create_mushaf_from_pages(pages)
        });
        BaseMushafEngine::new(mushaf)
    }

    // Function to reverse the visual representation of RTL text for terminal
    #[allow(unused)]
    fn reverse_for_terminal(text: &str) -> String {
        // Convert the string to an array of characters and reverse it
        let reversed: String = text.chars().rev().collect();
        reversed
    }

    #[test]
    fn faces_upwards_navigation() {
        let engine = setup_engine();

        let v = engine
            .navigate(
                15_f32,
                VersePosition::new(114, 1),
                Direction::Upwards,
                Default::default(),
            )
            .expect("expect `navigate` to succeed");
        assert_eq!(v.verse.sura, 112); // الإخلاص
        assert_eq!(v.verse.number, 4);

        // println!("Verse: {}, {}", v, reverse_for_terminal(QURAN_SURAS[v.verse.sura as usize]));

        let v = engine
            .navigate(
                30_f32,
                VersePosition::new(114, 1),
                Direction::Upwards,
                Default::default(),
            )
            .expect("expect `navigate` to succeed");
        assert_eq!(v.verse.sura, 109); // الكافرون
        assert_eq!(v.verse.number, 6);

        let v = engine
            .navigate(
                45_f32,
                VersePosition::new(114, 1),
                Direction::Upwards,
                Default::default(),
            )
            .expect("expect `navigate` to succeed");
        assert_eq!(v.verse.sura, 106); // قريش
        assert_eq!(v.verse.number, 4);

        let v = engine
            .navigate(
                15_f32,
                VersePosition::new(108, 1),
                Direction::Upwards,
                Default::default(),
            )
            .expect("expect `navigate` to succeed");
        assert_eq!(v.verse.sura, 106); // قريش
        assert_eq!(v.verse.number, 4);
    }

    #[test]
    fn faces_downwards_navigation() {
        let engine = setup_engine();

        let v = engine
            .navigate(
                15_f32,
                VersePosition::new(2, 6),
                Direction::Downwards,
                Default::default(),
            )
            .expect("expect `navigate` to succeed");
        assert_eq!(v.verse.sura, 2);
        assert_eq!(v.verse.number, 16);

        let v = engine
            .navigate(
                30_f32,
                VersePosition::new(2, 6),
                Direction::Downwards,
                Default::default(),
            )
            .expect("expect `navigate` to succeed");
        assert_eq!(v.verse.sura, 2);
        assert_eq!(v.verse.number, 24);

        let v = engine
            .navigate(
                15_f32,
                VersePosition::new(2, 17),
                Direction::Downwards,
                Default::default(),
            )
            .expect("expect `navigate` to succeed");
        assert_eq!(v.verse.sura, 2);
        assert_eq!(v.verse.number, 24);

        let v = engine
            .navigate(
                30_f32,
                VersePosition::new(4, 171),
                Direction::Downwards,
                Default::default(),
            )
            .expect("expect `navigate` to succeed");
        println!(
            "Verse: {:?}, {}",
            v,
            reverse_for_terminal(QURAN_SURAS[v.verse.sura as usize])
        );
        assert_eq!(v.verse.sura, 5); // قريش
        assert_eq!(v.verse.number, 2);
    }
}
