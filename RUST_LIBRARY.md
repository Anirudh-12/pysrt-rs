# pysrt-rs — Rust Library Documentation (`libsrt`)

> **Port Mortem 2026 — Track D: Python → Rust**  
> Comprehensive API guide and reference for using `pysrt-rs` (`libsrt`) as a standalone, memory-safe Rust crate.

---

## Overview

While `pysrt-rs` provides high-performance Python bindings (via PyO3) and a native CLI binary (`srt`), its core engine is architected as a standalone, zero-unsafe Rust library named **`libsrt`**.

> [!IMPORTANT]
> **100% Safe Rust Guarantee**: When the optional `python` feature is not enabled, the crate enforces `#![forbid(unsafe_code)]` at the root level ([src/lib.rs](file:///c:/Users/aksha/OneDrive/Documents/PYSRT_PORT/pysrt-rs/src/lib.rs#L1)).

> [!NOTE]
> **Dual Crate Type (`rlib` + `cdylib`)**: In [Cargo.toml](file:///c:/Users/aksha/OneDrive/Documents/PYSRT_PORT/pysrt-rs/Cargo.toml#L10-L12), the library target is configured with `crate-type = ["cdylib", "rlib"]`. This allows Cargo to link it as a standard Rust library (`rlib`) for downstream Rust crates while also compiling as a dynamic C-compatible shared library (`cdylib`) for Python extension modules.

---

## Getting Started

### 1. Add Dependency to `Cargo.toml`

To use the Rust library in your own project, add `pysrt-rs` to your `Cargo.toml`:

```toml
[dependencies]
# Depend by path (monorepo / local) or git repository
pysrt-rs = { path = "path/to/pysrt-rs" }

# Optional: rename the dependency to `pysrt` in your crate
pysrt = { package = "pysrt-rs", path = "path/to/pysrt-rs" }
```

### 2. Quickstart Example

Because the crate library target is named `libsrt`, import from `libsrt` in your Rust source code:

```rust
use libsrt::{open, SubRipFile, SubRipItem, SubRipTime};

fn main() -> libsrt::Result<()> {
    // 1. Open a subtitle file (auto-detecting UTF-8, UTF-32 BOMs, or fallback encodings)
    let mut srt = open("subtitles.srt", None)?;

    // 2. Shift all subtitle timestamps forward by 2 seconds and 500 milliseconds
    srt.shift(0, 0, 2, 500, None);

    // 3. Inspect individual subtitles
    for item in &srt.items {
        println!("{} --> {} | CPS: {:.1}", item.start, item.end, item.characters_per_second());
        println!("{}", item.text);
    }

    // 4. Save the modified subtitles back to disk (preserving detected line endings)
    srt.save(Some("subtitles_shifted.srt"))?;

    Ok(())
}
```

---

## Core API Reference

### Module Structure

```
libsrt
├── error       # SrtError enum and Result<T> alias
├── file        # SubRipFile document container and ErrorHandling mode
├── item        # SubRipItem subtitle entry and ItemIndex
└── time        # SubRipTime millisecond-precision timestamp and arithmetic
```

---

### 1. Top-Level Convenience Helpers

#### `pub fn open<P: AsRef<Path>>(path: P, encoding: Option<&str>) -> Result<SubRipFile>`
Opens an `.srt` file from the filesystem.
- If `encoding` is `None`, it automatically checks for UTF-32 LE/BE Byte-Order Marks (BOM), UTF-8, and falls back gracefully to `cp1252` (Windows-1252) when necessary.
- If an encoding string is provided (e.g., `"utf-8"`, `"windows-1252"`, `"utf-32-le"`), it decodes using `encoding_rs`.

#### `pub fn from_string(source: &str) -> Result<SubRipFile>`
Parses a subtitle file from an in-memory string slice using default strict error handling (`ErrorHandling::Raise`). Automatically guesses the line ending style (`\r\n`, `\n`, or `\r`).

---

### 2. `SubRipFile` (Subtitle Document Container)

`SubRipFile` represents an entire `.srt` subtitle file.

```rust
#[derive(Default, Clone, PartialEq, Eq)]
pub struct SubRipFile {
    pub items: Vec<SubRipItem>,
    pub eol: String,
    pub path: Option<String>,
    pub encoding: String,
}
```

#### Key Methods

| Method | Signature | Description |
|---|---|---|
| `open` | `open<P: AsRef<Path>>(path: P, encoding: Option<&str>) -> Result<Self>` | Reads and parses an `.srt` file from disk with optional explicit encoding. |
| `from_string` | `from_string(source: &str) -> Result<Self>` | Parses an `.srt` string with `ErrorHandling::Raise`. |
| `from_string_with_error_handling` | `from_string_with_error_handling(source: &str, mode: ErrorHandling) -> Result<Self>` | Parses an `.srt` string with customizable error recovery (`Raise`, `Log`, `Pass`). |
| `save` | `save<P: AsRef<Path>>(&self, path: Option<P>) -> Result<()>` | Writes subtitles to the specified path (or original path if `None`). |
| `shift` | `shift(&mut self, h: i64, m: i64, s: i64, ms: i64, ratio: Option<f64>)` | Shifts all item timestamps by offset and optional frame rate scaling ratio. |
| `clean_indexes` | `clean_indexes(&mut self)` | Sorts all subtitles chronologically by start time and re-numbers indexes sequentially from 1. |
| `slice` | `slice(&self, start: usize, end: usize) -> Vec<SubRipItem>` | Returns a cloned slice of subtitle items within the index range. |
| `at` | `at(&self, time: SubRipTime) -> Vec<&SubRipItem>` | Finds all subtitles active at the given timestamp. |
| `text` | `text(&self) -> String` | Renders the entire `.srt` document back into a String using stored line endings. |
| `write_into` | `write_into<W: Write>(&self, writer: &mut W) -> Result<()>` | Renders subtitles directly into any `std::io::Write` stream. |

---

### 3. `SubRipItem` (Subtitle Entry)

`SubRipItem` represents a single subtitle block in an `.srt` file.

```rust
#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct SubRipItem {
    pub index: ItemIndex,
    pub start: SubRipTime,
    pub end: SubRipTime,
    pub text: String,
    pub position: String,
}
```

#### Key Methods

| Method | Return Type | Description |
|---|---|---|
| `new(...)` | `Self` | Constructs a new `SubRipItem` from index, start/end times, text, and optional position coordinates. |
| `duration()` | `SubRipTime` | Calculates the duration of the subtitle item (`end - start`). |
| `text_without_tags()` | `String` | Strips out HTML/XML styling tags (e.g., `<i>`, `<b>`, `<font color="...">`). |
| `characters_per_second()` | `f64` | Returns reading speed in characters per second (CPS), excluding newline characters and formatting tags. |
| `shift(...)` | `()` | Shifts start and end timestamps by hours, minutes, seconds, milliseconds, and frame rate ratio. |

#### `ItemIndex` Enum
Handles subtitle numbering flexibly, matching original Python `pysrt` semantics:
```rust
pub enum ItemIndex {
    Int(i32),      // Standard numeric index (1, 2, 3...)
    Str(String),   // Non-standard string index preserved from malformed files
    None,          // Unindexed subtitle block
}
```

---

### 4. `SubRipTime` (Millisecond-Precision Timestamp)

`SubRipTime` represents timestamps internally as an ordinal number of milliseconds from `00:00:00,000`, guaranteeing exact integer arithmetic without floating-point drift.

```rust
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubRipTime {
    pub ordinal: i64, // Total milliseconds
}
```

#### Constructors & Parsing
- **`SubRipTime::new(hours, minutes, seconds, milliseconds)`**: Constructs timestamp from components.
- **`SubRipTime::from_ordinal(ordinal_ms)`**: Constructs timestamp from total milliseconds.
- **`SubRipTime::from_string(source)`**: Parses standard SRT time strings (`"HH:MM:SS,mmm"` or `"HH:MM:SS.mmm"`).

#### Getters and Setters
- **Getters**: `.hours()`, `.minutes()`, `.seconds()`, `.milliseconds()`
- **Setters**: `.set_hours(val)`, `.set_minutes(val)`, `.set_seconds(val)`, `.set_milliseconds(val)`

#### Operator Overloading & Math
`SubRipTime` implements standard Rust mathematical operators:
- **`Add` / `Sub`**: Add or subtract two `SubRipTime` instances, or add/subtract integer milliseconds.
- **`Mul`**: Scale a timestamp by a floating-point factor (`f64`) or integer factor (`i64`), useful for frame rate conversions.
- **Comparison**: Fully implements `PartialOrd` and `Ord` for chronological sorting.

---

### 5. Error Handling (`SrtError` & `Result<T>`)

All fallible operations return `libsrt::Result<T>`, which wraps `libsrt::SrtError`.

```rust
#[derive(Error, Debug)]
pub enum SrtError {
    #[error("Invalid time string: {0}")]
    InvalidTimeString(String),

    #[error("Invalid subtitle item: {0}")]
    InvalidItem(String),

    #[error("Invalid subtitle index: {0}")]
    InvalidIndex(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encoding error: {0}")]
    Encoding(String),
}
```

#### Error Handling Modes (`ErrorHandling`)
When parsing subtitle strings or files that may contain syntax errors, you can specify how the parser behaves:

```rust
pub enum ErrorHandling {
    Pass = 0,   // Silently drop malformed items and continue parsing
    Log = 1,    // Log warnings to stderr for malformed items and continue parsing
    Raise = 2,  // Abort immediately and return SrtError upon encountering malformed items
}
```

---

## Practical Examples

### Example 1: Frame Rate Conversion (23.976 fps → 25.0 fps)

When converting subtitles between video releases with different frame rates, timestamps must be scaled proportionally:

```rust
use libsrt::{open, Result};

fn main() -> Result<()> {
    let mut srt = open("movie_23976.srt", None)?;

    // Calculate frame rate conversion ratio
    let old_fps = 23.976_f64;
    let new_fps = 25.0_f64;
    let ratio = new_fps / old_fps;

    // Shift with 0 time offsets and the calculated scaling ratio
    srt.shift(0, 0, 0, 0, Some(ratio));

    // Save converted subtitle file
    srt.save(Some("movie_25000.srt"))?;
    Ok(())
}
```

---

### Example 2: Filtering Subtitles by Reading Speed (CPS)

Broadcasting standards often recommend keeping subtitle reading speed below 20 characters per second (CPS). You can inspect and flag high-CPS items:

```rust
use libsrt::{open, Result};

fn main() -> Result<()> {
    let srt = open("feature_film.srt", None)?;

    println!("Checking for subtitles exceeding 20.0 CPS...");
    for item in &srt.items {
        let cps = item.characters_per_second();
        if cps > 20.0 {
            println!(
                "[{}] CPS {:.1}: {} --> {} | Text: {:?}",
                item.index,
                cps,
                item.start,
                item.end,
                item.text_without_tags()
            );
        }
    }

    Ok(())
}
```

---

### Example 3: Parsing Malformed Files with Error Recovery

If you are dealing with noisy subtitle files (e.g., OCR artifacts or corrupted lines), use `ErrorHandling::Log` or `ErrorHandling::Pass`:

```rust
use libsrt::{ErrorHandling, SubRipFile, Result};

fn main() -> Result<()> {
    let noisy_srt = "\
1
00:00:01,000 --> 00:00:03,000
Valid subtitle item

2
INVALID TIMESTAMP LINE
Corrupted subtitle item that should be skipped

3
00:00:05,000 --> 00:00:07,500
Another valid subtitle item
";

    // Parse while logging skipped malformed items to stderr instead of failing
    let srt = SubRipFile::from_string_with_error_handling(noisy_srt, ErrorHandling::Log)?;

    assert_eq!(srt.items.len(), 2);
    println!("Successfully parsed {} valid items!", srt.items.len());
    Ok(())
}
```

---

### Example 4: Creating Subtitle Files from Scratch & In-Place Cleaning

You can programmatically generate `.srt` files and enforce strict chronological order and numbering:

```rust
use libsrt::{ItemIndex, SubRipFile, SubRipItem, SubRipTime, Result};

fn main() -> Result<()> {
    let item1 = SubRipItem::new(
        ItemIndex::Int(10), // Out-of-order index
        SubRipTime::new(0, 1, 30, 0),
        SubRipTime::new(0, 1, 33, 500),
        "Second line in dialogue".to_string(),
        String::new(),
    );

    let item2 = SubRipItem::new(
        ItemIndex::Int(5),
        SubRipTime::new(0, 0, 15, 0),
        SubRipTime::new(0, 0, 18, 0),
        "First line in dialogue".to_string(),
        String::new(),
    );

    let mut srt = SubRipFile::new(vec![item1, item2], Some("\n".to_string()), None, None);

    // clean_indexes() sorts by start timestamp and re-numbers sequentially from 1
    srt.clean_indexes();

    assert_eq!(srt.items[0].index.as_i32(), 1);
    assert_eq!(srt.items[0].text, "First line in dialogue");
    assert_eq!(srt.items[1].index.as_i32(), 2);
    assert_eq!(srt.items[1].text, "Second line in dialogue");

    srt.save(Some("generated.srt"))?;
    Ok(())
}
```

---

## Encoding & Line Ending Support

### Automatic Character Encoding Detection
`libsrt` leverages `encoding_rs` for fast, correct text decoding:
1. **UTF-32 LE / BE**: Automatically detects 4-byte Byte-Order Marks (`0xFF 0xFE 0x00 0x00` and `0x00 0x00 0xFE 0xFF`) and decodes UTF-32 correctly.
2. **UTF-8 / CP1252**: When opening files without an explicit encoding parameter (`None`), UTF-8 is attempted first; if invalid UTF-8 byte sequences are detected, it falls back cleanly to `cp1252` (Windows-1252), mirroring Python `pysrt` behavior.
3. **Explicit Encodings**: Any standard label supported by `encoding_rs` can be passed to `libsrt::open(path, Some("encoding_label"))`.

### Line Ending Preservation
When parsing subtitle documents, `SubRipFile::guess_eol()` identifies the newline convention (`\r\n` for Windows CRLF, `\n` for Unix LF, or `\r` for old macOS). When calling `.save()` or `.text()`, the original line ending convention is preserved automatically.
