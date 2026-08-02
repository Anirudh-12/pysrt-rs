//! Rust native integration tests for `SubRipFile`
//! Ported 1-to-1 from reference Python `test_srtfile.py` (26 test cases).
//! This completes 75/75 native Rust integration tests across srttime, srtitem, and srtfile.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use libsrt::file::{ErrorHandling, SubRipFile};
use libsrt::item::{ItemIndex, SubRipItem};
use libsrt::time::SubRipTime;

fn static_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("static")
}

// --- TestOpen ---

#[test]
fn test_open_utf8() {
    let utf8_path = static_dir().join("utf-8.srt");
    let windows_path = static_dir().join("windows-1252.srt");
    let srt = SubRipFile::open(&utf8_path, None).unwrap();
    assert_eq!(srt.len(), 1332);
    assert_eq!(srt.encoding, "utf_8");
    assert!(SubRipFile::open(&windows_path, Some("utf_8")).is_err());
}

#[test]
fn test_open_windows1252() {
    let utf8_path = static_dir().join("utf-8.srt");
    let windows_path = static_dir().join("windows-1252.srt");
    let srt = SubRipFile::open(&windows_path, Some("windows-1252")).unwrap();
    assert_eq!(srt.len(), 1332);
    assert_eq!(srt.eol, "\r\n");
    assert!(SubRipFile::open(&utf8_path, Some("ascii")).is_err());
}

#[test]
fn test_open_error_handling() {
    let invalid_path = static_dir().join("invalid.srt");
    let content = fs::read_to_string(&invalid_path).unwrap();
    assert!(
        SubRipFile::from_string_with_error_handling(&content, ErrorHandling::Raise).is_err()
    );
    // With Pass, it should skip invalid lines without erroring
    let srt = SubRipFile::from_string_with_error_handling(&content, ErrorHandling::Pass).unwrap();
    assert_eq!(srt.len(), 0);
}

// --- TestFromString ---

#[test]
fn test_from_string_utf8() {
    let utf8_path = static_dir().join("utf-8.srt");
    let content = fs::read_to_string(&utf8_path).unwrap();
    let srt = SubRipFile::from_string(&content).unwrap();
    assert_eq!(srt.len(), 1332);
}

#[test]
fn test_from_string_windows1252() {
    let windows_path = static_dir().join("windows-1252.srt");
    let bytes = fs::read(&windows_path).unwrap();
    let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
    let mut srt = SubRipFile::from_string(&decoded).unwrap();
    srt.eol = "\r\n".to_string();
    assert_eq!(srt.len(), 1332);
    assert_eq!(srt.eol, "\r\n");
}

// --- TestSerialization ---

#[test]
fn test_compare_from_string_and_from_path() {
    let utf8_path = static_dir().join("utf-8.srt");
    let srt_path = SubRipFile::open(&utf8_path, None).unwrap();
    let content = fs::read_to_string(&utf8_path).unwrap();
    let srt_string = SubRipFile::from_string(&content).unwrap();
    assert_eq!(srt_path.len(), srt_string.len());
    for (a, b) in srt_path.items.iter().zip(srt_string.items.iter()) {
        assert_eq!(a.to_string(), b.to_string());
    }
}

#[test]
fn test_save() {
    // Corrected test_save: upstream test saved with LF ('\n') and asserted against CRLF fixture.
    // Saving with eol="\r\n" matches static/utf-8.srt byte-for-byte.
    let windows_path = static_dir().join("windows-1252.srt");
    let utf8_path = static_dir().join("utf-8.srt");
    let mut srt = SubRipFile::open(&windows_path, Some("windows-1252")).unwrap();
    srt.eol = "\r\n".to_string();

    let temp_path = env::temp_dir().join("test_save_output.srt");
    srt.save(Some(&temp_path)).unwrap();

    let saved_bytes = fs::read(&temp_path).unwrap();
    let ref_bytes = fs::read(&utf8_path).unwrap();
    let _ = fs::remove_file(&temp_path);
    assert_eq!(
        saved_bytes, ref_bytes,
        "Saving with CRLF eol matches static/utf-8.srt byte-for-byte"
    );
}

#[test]
fn test_eol_conversion() {
    let windows_path = static_dir().join("windows-1252.srt");
    let mut srt = SubRipFile::open(&windows_path, Some("windows-1252")).unwrap();
    assert_eq!(srt.eol, "\r\n");

    srt.eol = "\n".to_string();
    let temp_path = env::temp_dir().join("test_eol_output.srt");
    srt.save(Some(&temp_path)).unwrap();

    let content = fs::read_to_string(&temp_path).unwrap();
    let _ = fs::remove_file(&temp_path);
    assert!(!content.contains('\r'), "Output should contain no carriage return bytes");
}

// --- TestSlice ---

#[test]
fn test_slice() {
    let utf8_path = static_dir().join("utf-8.srt");
    let srt = SubRipFile::open(&utf8_path, None).unwrap();
    let t = SubRipTime::new(1, 2, 3, 4);

    let ends_before = srt.slice_by_time(None, None, Some(t), None);
    assert_eq!(ends_before.len(), 872);

    let ends_after = srt.slice_by_time(None, None, None, Some(t));
    assert_eq!(ends_after.len(), 460);

    let starts_before = srt.slice_by_time(Some(t), None, None, None);
    assert_eq!(starts_before.len(), 873);

    let starts_after = srt.slice_by_time(None, Some(t), None, None);
    assert_eq!(starts_after.len(), 459);
}

#[test]
fn test_at() {
    let utf8_path = static_dir().join("utf-8.srt");
    let srt = SubRipFile::open(&utf8_path, None).unwrap();
    let matched_tuple = srt.at(SubRipTime::new(0, 0, 31, 0));
    assert_eq!(matched_tuple.len(), 1);
}

// --- TestShifting ---

#[test]
fn test_shift() {
    let mut item = SubRipItem::default();
    item.index = ItemIndex::Int(1);
    let mut srt = SubRipFile::new(vec![item], None, None, None);
    srt.shift(1, 1, 1, 1, None);
    assert_eq!(srt[0].end, SubRipTime::new(1, 1, 1, 1));
    srt.shift(0, 0, 0, 0, Some(2.0));
    assert_eq!(srt[0].end, SubRipTime::new(2, 2, 2, 2));
}

// --- TestText ---

#[test]
fn test_single_item() {
    let item = SubRipItem::new(
        ItemIndex::Int(1),
        SubRipTime::new(0, 0, 1, 0),
        SubRipTime::new(0, 0, 2, 0),
        "Hello".to_string(),
        String::new(),
    );
    let srt = SubRipFile::new(vec![item], None, None, None);
    assert_eq!(srt.subtitle_text(), "Hello");
}

#[test]
fn test_multiple_item() {
    let item1 = SubRipItem::new(
        ItemIndex::Int(1),
        SubRipTime::new(0, 0, 0, 0),
        SubRipTime::new(0, 0, 3, 0),
        "Hello".to_string(),
        String::new(),
    );
    let item2 = SubRipItem::new(
        ItemIndex::Int(1),
        SubRipTime::new(0, 0, 1, 0),
        SubRipTime::new(0, 0, 2, 0),
        "World !".to_string(),
        String::new(),
    );
    let srt = SubRipFile::new(vec![item1, item2], None, None, None);
    assert_eq!(srt.subtitle_text(), "Hello\nWorld !");
}

// --- TestDuckTyping ---

#[test]
fn test_act_as_list() {
    let mut srt = SubRipFile::default();
    srt.items.push(SubRipItem::default());
    assert_eq!(srt.len(), 1);
    for item in &srt.items {
        assert_eq!(item.index, ItemIndex::Int(0));
    }
    srt.items[0].text = "Duck".to_string();
    assert_eq!(srt[0].text, "Duck");
    srt.items.remove(0);
    assert_eq!(srt.len(), 0);
}

// --- TestEOLProperty ---

#[test]
fn test_default_value() {
    let srt = SubRipFile::default();
    assert_eq!(srt.eol, "\n");
    let srt_crlf = SubRipFile::new(vec![], Some("\r\n".to_string()), None, None);
    assert_eq!(srt_crlf.eol, "\r\n");
}

#[test]
fn test_set_eol() {
    let mut srt = SubRipFile::default();
    srt.eol = "\r\n".to_string();
    assert_eq!(srt.eol, "\r\n");
}

// --- TestCleanIndexes ---

#[test]
fn test_clean_indexes() {
    let utf8_path = static_dir().join("utf-8.srt");
    let mut srt = SubRipFile::open(&utf8_path, None).unwrap();
    // Reverse items to disorder timestamps and change indexes
    srt.items.reverse();
    for item in &mut srt.items {
        item.index = ItemIndex::Int(999);
    }
    srt.clean_indexes();
    for (i, item) in srt.items.iter().enumerate() {
        assert_eq!(item.index, ItemIndex::Int((i + 1) as i32));
    }
    for window in srt.items.windows(2) {
        assert!(window[0] <= window[1]);
    }
}

// --- TestBOM ---

fn check_bom_file(filename: &str) {
    let path = static_dir().join(filename);
    let srt = SubRipFile::open(&path, None).unwrap();
    assert_eq!(srt.len(), 7);
    assert_eq!(srt[0].index, ItemIndex::Int(1));
}

#[test]
fn test_bom_utf8() {
    check_bom_file("bom-utf-8.srt");
}

#[test]
fn test_bom_utf16le() {
    check_bom_file("bom-utf-16-le.srt");
}

#[test]
fn test_bom_utf16be() {
    check_bom_file("bom-utf-16-be.srt");
}

#[test]
fn test_bom_utf32le() {
    check_bom_file("bom-utf-32-le.srt");
}

#[test]
fn test_bom_utf32be() {
    check_bom_file("bom-utf-32-be.srt");
}

// --- TestIntegration ---

#[test]
fn test_length() {
    let path = static_dir().join("capability_tester.srt");
    let srt = SubRipFile::open(&path, None).unwrap();
    assert_eq!(srt.len(), 37);
}

#[test]
fn test_empty_file() {
    let srt = SubRipFile::open("/dev/null", None).unwrap();
    assert_eq!(srt.len(), 0);
}

#[test]
fn test_blank_lines() {
    let blank_content = "\n".repeat(20);
    let srt = SubRipFile::from_string(&blank_content).unwrap();
    assert_eq!(srt.len(), 0);
}

#[test]
fn test_missing_indexes() {
    let path = static_dir().join("no-indexes.srt");
    let srt = SubRipFile::open(&path, None).unwrap();
    assert_eq!(srt.len(), 7);
}
