# Reverse Navigation (`reverse_navigate`)

`reverse_navigate` moves through the Mushaf by a number of **lines**, like `navigate`, but steps through verses using `previous_verse` instead of `next_verse`.

It is **not** the same as calling `navigate` with the opposite `Direction`.

## Two independent knobs

| Knob | What it controls |
|------|------------------|
| **`Direction`** | Which way the Quran is read globally: `Downwards` (1 → 114) or `Upwards` (114 → 1). This affects bounds, cycling, and which sura comes next at sura boundaries. |
| **`reverse_navigate` vs `navigate`** | Which neighbor verse to visit next along that reading order: `next_verse` (forward) or `previous_verse` (backward). |

`reverse_navigate` still takes a `direction` argument and **obeys it** — bounds, cycling, and sura-boundary rules are the same as `navigate`. Only the step function changes.

## Why flipping `Direction` is not enough

From **1:2**, moving **5 lines** with default settings:

| Method | `Direction` | Lands on |
|--------|-------------|----------|
| `navigate` | `Downwards` | **1:6** |
| `navigate` | `Upwards` | **1:6** |
| `reverse_navigate` | `Downwards` | **1:1** |

Both `navigate` calls walk **forward** through verses (1:3, 1:4, …) regardless of `Direction`. `Direction` does not mean “increase or decrease the verse number” within a sura.

From **2:1**, moving **5 lines** `Downwards`:

| Method | Lands on |
|--------|----------|
| `navigate` | **2:2** (into the next verses of Al-Baqarah) |
| `reverse_navigate` | **1:7** (back into the previous sura) |

Here `reverse_navigate` crosses into the **previous** sura along the downward reading order — not by flipping direction.

From **78:1**, moving **10 lines** `Downwards`:

| Method | Lands on |
|--------|----------|
| `navigate` | **78:19** (forward in An-Naba) |
| `reverse_navigate` | **77:36** (backward into Al-Mursalat) |

## Mental model

```
Direction         = which highway you are on (toward 114 or toward 1)
navigate          = drive forward on that highway  (next_verse)
reverse_navigate  = drive backward on that highway (previous_verse)
```

Flipping `Direction` switches highways. `reverse_navigate` reverses your gear on the **same** highway.

## API

Same parameters as `navigate`:

```rust
use mushaf_engine::{
    BaseMushafEngine, Direction, IMushafEngine, NavigationSettings, VersePosition,
};

let result = engine.reverse_navigate(
    5.0,
    VersePosition::new(1, 2),
    Direction::Downwards,
    NavigationSettings::builder(),
)?;
```

Returns the same `NavigationResult`: target verse, distance moved, overflow, end-of-page/sura hints, and cycle info.

## When to use which

| Goal | Use |
|------|-----|
| Normal reading progress (memorization plans, page turns) | `navigate` |
| Undo / go back N lines from a bookmark | `reverse_navigate` |
| Walk verse-by-verse forward | `next_verse` |
| Walk verse-by-verse backward | `previous_verse` |

## Round-trip behavior

Moving forward then backward (or the reverse) with the **same line count** often returns close to the start, but not always symmetric — line accounting includes partial verse overflow the same way as `navigate`.

```rust
use mushaf_engine::{
    BaseMushafEngine, Direction, IMushafEngine, NavigationSettings, VersePosition,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = BaseMushafEngine::king_fahad()?;
    let start = VersePosition::new(2, 50);
    let lines = 20.0;
    let settings = NavigationSettings::builder();

    let after_reverse = engine.reverse_navigate(
        lines,
        start,
        Direction::Downwards,
        settings,
    )?;
    // after_reverse.verse → 2:37

    let back = engine.navigate(
        lines,
        &after_reverse.verse,
        Direction::Downwards,
        settings,
    )?;
    // back.verse → 2:50 (returns to start)

    assert_eq!(back.verse.sura, 2);
    assert_eq!(back.verse.number, 50);
    Ok(())
}
```

## Related

- [README — Quick start](../README.md#quick-start) — forward line-based navigation
- [`IMushafEngine`](../src/mushaf_engine/i_mushaf_engine.rs) — trait API
- Tests: `src/mushaf_engine/base_mushaf_engine_tests.rs` (`reverse_navigate_*`)
