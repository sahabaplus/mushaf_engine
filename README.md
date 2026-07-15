# Mushaf Engine (Rust)

Line-based Quran Mushaf navigation with cycling bounds, sura-header control, and rich navigation results.

Based on the King Fahad Quran Printing Complex Mushaf edition (15 lines per page).

## Features

- **Line-based navigation** — move by fractional lines via `navigate` / `reverse_navigate`
- **Cycling bounds** — restrict to a verse range and optionally loop with an iteration limit
- **Sura header control** — include or exclude bismillah / title lines from distance math
- **Rich results** — target verse, overflow, end-of-page / end-of-sura hints, cycle info
- **Verse lookup & distance** — find verses, resolve positions, `calculate_lines` between two points
- **Bundled King Fahad data** — load with `BaseMushafEngine::king_fahad()` (default feature)

## Installation

```toml
[dependencies]
mushaf_engine = "0.2"
```

### Features

| Feature | Default | Description |
|---------|---------|-------------|
| `king_fahad_mushaf` | yes | JSON loader + bundled `data/king_fahad_mushaf.json` |
| `colored_output` | no | Colored `Display` for verses / navigation results |

## Quick start

```rust
use mushaf_engine::{
    BaseMushafEngine, Direction, IMushafEngine, NavigationSettings, VersePosition,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = BaseMushafEngine::king_fahad()?;

    // Navigate 15 lines from Sura 5, Verse 3 downwards
    let result = engine.navigate(
        15.0,
        VersePosition::new(5, 3),
        Direction::Downwards,
        NavigationSettings::builder(),
    )?;

    println!("{result}");
    Ok(())
}
```

## Reverse navigation

`reverse_navigate` moves by lines like `navigate`, but steps with `previous_verse` instead of `next_verse`.

It is **not** the same as calling `navigate` with the opposite `Direction`. See **[docs/reverse-navigate.md](docs/reverse-navigate.md)**.

```rust
use mushaf_engine::{
    BaseMushafEngine, Direction, IMushafEngine, NavigationSettings, VersePosition,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = BaseMushafEngine::king_fahad()?;

    let result = engine.reverse_navigate(
        5.0,
        VersePosition::new(1, 2),
        Direction::Downwards,
        NavigationSettings::builder(),
    )?;

    println!("{result}");
    Ok(())
}
```

| From 1:2, 5 lines | Lands on |
|-------------------|----------|
| `navigate` (`Downwards`) | 1:6 |
| `navigate` (`Upwards`) | 1:6 |
| `reverse_navigate` (`Downwards`) | 1:1 |

## Navigation settings

```rust
use mushaf_engine::{NavigationSettings, VersePosition};

// Cycle Al-Baqarah up to 3 times, exclude sura headers from line math
let settings = NavigationSettings::builder()
    .ignore_sura_header(true)
    .iteration_limit(3)
    .upper_bound(VersePosition::new(2, 1))
    .lower_bound(VersePosition::new(2, 286));
```

### Bounds & cycling

- **Inclusive** (`upper < lower`): navigate only inside that range
- **Excluding** (`upper > lower`): navigate everywhere *except* that range
- **`iteration_limit`**: `0` = stop at the edge; `n` = allow `n` full cycles

```rust
use mushaf_engine::{
    BaseMushafEngine, Direction, IMushafEngine, NavigationSettings, VersePosition,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = BaseMushafEngine::king_fahad()?;

    // Juz' Amma, up to 5 cycles
    let settings = NavigationSettings::builder()
        .iteration_limit(5)
        .upper_bound(VersePosition::new(78, 1))
        .lower_bound(VersePosition::new(114, 6));

    let result = engine.navigate(
        1000.0,
        VersePosition::new(78, 1),
        Direction::Downwards,
        settings,
    )?;

    println!("{result}");
    Ok(())
}
```

### Distance between verses

```rust
use mushaf_engine::{
    BaseMushafEngine, Direction, IMushafEngine, NavigationSettings, VersePosition,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = BaseMushafEngine::king_fahad()?;

    let lines = engine.calculate_lines(
        VersePosition::new(2, 255), // Ayat al-Kursi
        VersePosition::new(3, 200), // end of Al-Imran
        Direction::Downwards,
        NavigationSettings::builder(),
    )?;

    println!("Distance: {lines} lines");
    Ok(())
}
```

Sura 9 (At-Tawbah) has no bismillah — only a 1-line title instead of the usual 2.

## Data format

King Fahad JSON is an array of pages; each page is an array of verses:

```json
[
  [
    { "sura": 1, "ayah": 1, "lines": 0.8, "y": 1, "x": 0.8 }
  ]
]
```

| Field | Meaning |
|-------|---------|
| `sura` | Sura number (1–114) |
| `ayah` | Verse within the sura |
| `lines` | Lines spanned (may be fractional) |
| `y` | Line on the page (1–15) |
| `x` | Horizontal position on the line (0.0–1.0) |

Load from a custom path:

```rust
use std::sync::Arc;
use mushaf_engine::{BaseMushafEngine, KingFahadMushaf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mushaf = KingFahadMushaf::from_file("path/to/mushaf.json")?;
    let engine = BaseMushafEngine::new(Arc::new(mushaf));
    Ok(())
}
```

## API overview

| Type | Role |
|------|------|
| `BaseMushafEngine` | Main engine (`IMushafEngine`) |
| `NavigationSettings` | Builder for bounds, cycling, header handling |
| `NavigationBounds` | Range + iteration limit |
| `VersePosition` | `(sura, verse)` identity |
| `Direction` | `Downwards` (1→114) or `Upwards` (114→1) |
| `NavigationResult` | Target verse, overflow, page/sura ends, cycles |
| `Mushaf` / `Verse` / `Page` | Mushaf model |
| `KingFahadMushaf` | JSON → `Mushaf` |

Core `IMushafEngine` methods: `navigate`, `reverse_navigate`, `calculate_lines`, `next_verse`, `previous_verse`, `find_verse`, `find_verse_location`, `get_sura_info`, and bound helpers.

```bash
cargo doc --open
```

## Development

```bash
cargo test
cargo test --doc
cargo doc --open
```

## Links

- Repository: [sahabaplus/mushaf_engine](https://github.com/sahabaplus/mushaf_engine)
- Reverse navigation guide: [docs/reverse-navigate.md](docs/reverse-navigate.md)

## License

MIT — see [LICENSE](./LICENSE).
