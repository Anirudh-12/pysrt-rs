//! Rust native integration tests for `SubRipItem`
//! Ported 1-to-1 from reference Python `test_srtitem.py` (all test cases).

use libsrt::item::{ItemIndex, SubRipItem};
use libsrt::time::SubRipTime;

fn sample_item() -> SubRipItem {
    let mut item = SubRipItem::new(
        ItemIndex::Int(1),
        SubRipTime::default(),
        SubRipTime::default(),
        "Hello world !".to_string(),
        String::new(),
    );
    item.shift(0, 1, 0, 0, None);
    item.end.shift(0, 0, 20, 0, None);
    item
}

// --- TestAttributes ---
#[test]
fn test_has_id() {
    let item = SubRipItem::default();
    assert_eq!(item.index, ItemIndex::Int(0));
}

#[test]
fn test_has_content() {
    let item = SubRipItem::default();
    assert_eq!(item.text, "");
}

#[test]
fn test_has_start() {
    let item = SubRipItem::default();
    assert_eq!(item.start, SubRipTime::default());
}

#[test]
fn test_has_end() {
    let item = SubRipItem::default();
    assert_eq!(item.end, SubRipTime::default());
}

// --- TestDuration ---
#[test]
fn test_duration() {
    let item = sample_item();
    assert_eq!(item.duration(), SubRipTime::new(0, 0, 20, 0));
}

// --- TestCPS ---
#[test]
fn test_characters_per_second() {
    let item = sample_item();
    assert_eq!(item.characters_per_second(), 0.65);
}

#[test]
fn test_text_change() {
    let mut item = sample_item();
    item.text = "Hello world !\nHello world again !".to_string();
    assert_eq!(item.characters_per_second(), 1.6);
}

#[test]
fn test_zero_duration() {
    let mut item = sample_item();
    item.start.shift(0, 0, 20, 0, None);
    assert_eq!(item.characters_per_second(), 0.0);
}

#[test]
fn test_tags() {
    let mut item = sample_item();
    item.text = "<b>bold</b>, <i>italic</i>, <u>underlined</u>\n\
                 <font color=\"#ff0000\">red text</font>\
                 , <b>one,<i> two,<u> three</u></i></b>"
        .to_string();
    assert_eq!(item.characters_per_second(), 2.45);
}

// --- TestTagRemoval ---
#[test]
fn test_italics_tag() {
    let mut item = sample_item();
    item.text = "<i>Hello world !</i>".to_string();
    assert_eq!(item.text_without_tags(), "Hello world !");
}

#[test]
fn test_bold_tag() {
    let mut item = sample_item();
    item.text = "<b>Hello world !</b>".to_string();
    assert_eq!(item.text_without_tags(), "Hello world !");
}

#[test]
fn test_underline_tag() {
    let mut item = sample_item();
    item.text = "<u>Hello world !</u>".to_string();
    assert_eq!(item.text_without_tags(), "Hello world !");
}

#[test]
fn test_color_tag() {
    let mut item = sample_item();
    item.text = "<font color=\"#ff0000\">Hello world !</font>".to_string();
    assert_eq!(item.text_without_tags(), "Hello world !");
}

#[test]
fn test_all_tags() {
    let mut item = sample_item();
    item.text = "<b>Bold</b>, <i>italic</i>, <u>underlined</u>\n\
                 <font color=\"#ff0000\">red text</font>\
                 , <b>one,<i> two,<u> three</u></i></b>."
        .to_string();
    assert_eq!(
        item.text_without_tags(),
        "Bold, italic, underlined\nred text, one, two, three."
    );
}

// --- TestShifting ---
#[test]
fn test_shift_up() {
    let mut item = sample_item();
    item.shift(1, 2, 3, 4, None);
    assert_eq!(item.start, SubRipTime::new(1, 3, 3, 4));
    assert_eq!(item.end, SubRipTime::new(1, 3, 23, 4));
    assert_eq!(item.duration(), SubRipTime::new(0, 0, 20, 0));
    assert_eq!(item.characters_per_second(), 0.65);
}

#[test]
fn test_shift_down() {
    let mut item = sample_item();
    item.shift(5, 0, 0, 0, None);
    item.shift(-1, -2, -3, -4, None);
    assert_eq!(item.start, SubRipTime::new(3, 58, 56, 996));
    assert_eq!(item.end, SubRipTime::new(3, 59, 16, 996));
    assert_eq!(item.duration(), SubRipTime::new(0, 0, 20, 0));
    assert_eq!(item.characters_per_second(), 0.65);
}

#[test]
fn test_shift_by_ratio() {
    let mut item = sample_item();
    item.shift(0, 0, 0, 0, Some(2.0));
    assert_eq!(item.start, SubRipTime::new(0, 2, 0, 0));
    assert_eq!(item.end, SubRipTime::new(0, 2, 40, 0));
    assert_eq!(item.duration(), SubRipTime::new(0, 0, 40, 0));
    assert_eq!(item.characters_per_second(), 0.325);
}

// --- TestOperators ---
#[test]
fn test_cmp() {
    let item = sample_item();
    assert_eq!(item, item.clone());
}

// --- TestSerialAndParsing ---
#[test]
fn test_serialization() {
    let item = sample_item();
    let string = "1\n00:01:00,000 --> 00:01:20,000\nHello world !\n";
    assert_eq!(item.to_string(), string);
}

#[test]
fn test_from_string() {
    let item = sample_item();
    let string = "1\n00:01:00,000 --> 00:01:20,000\nHello world !\n";
    let parsed = SubRipItem::from_string(string).unwrap();
    assert_eq!(parsed, item);
    let bad_string = "foobar";
    assert!(SubRipItem::from_string(bad_string).is_err());
}

#[test]
fn test_coordinates() {
    let item = sample_item();
    let coordinates =
        "1\n00:01:00,000 --> 00:01:20,000 X1:000 X2:000 Y1:050 Y2:100\nHello world !\n";
    let parsed = SubRipItem::from_string(coordinates).unwrap();
    assert_eq!(parsed.index, item.index);
    assert_eq!(parsed.start, item.start);
    assert_eq!(parsed.end, item.end);
    assert_eq!(parsed.text, item.text);
    assert_eq!(parsed.position, "X1:000 X2:000 Y1:050 Y2:100");
}

#[test]
fn test_vtt_positioning() {
    let vtt = "1\n00:01:00,000 --> 00:01:20,000 D:vertical A:start L:12%\nHello world !\n";
    let parsed = SubRipItem::from_string(vtt).unwrap();
    assert_eq!(parsed.position, "D:vertical A:start L:12%");
    assert_eq!(parsed.index, ItemIndex::Int(1));
    assert_eq!(parsed.text, "Hello world !");
}

#[test]
fn test_idempotence() {
    let vtt = "1\n00:01:00,000 --> 00:01:20,000 D:vertical A:start L:12%\nHello world !\n";
    let parsed_vtt = SubRipItem::from_string(vtt).unwrap();
    assert_eq!(parsed_vtt.to_string(), vtt);

    let coordinates =
        "1\n00:01:00,000 --> 00:01:20,000 X1:000 X2:000 Y1:050 Y2:100\nHello world !\n";
    let parsed_coords = SubRipItem::from_string(coordinates).unwrap();
    assert_eq!(parsed_coords.to_string(), coordinates);
}

#[test]
fn test_dots() {
    let item = sample_item();
    let dots = "1\n00:01:00.000 --> 00:01:20.000\nHello world !\n";
    let parsed = SubRipItem::from_string(dots).unwrap();
    assert_eq!(parsed, item);
}

#[test]
fn test_paring_error() {
    let bad = "1\n00:01:00,000 -> 00:01:20,000 X1:000 X2:000 Y1:050 Y2:100\nHello world !\n";
    assert!(SubRipItem::from_string(bad).is_err());
}

#[test]
fn test_string_index() {
    let string_index = "foo\n00:01:00,000 --> 00:01:20,000\nHello !\n";
    let parsed = SubRipItem::from_string(string_index).unwrap();
    assert_eq!(parsed.index, ItemIndex::Str("foo".to_string()));
    assert_eq!(parsed.text, "Hello !");
}

#[test]
fn test_no_index() {
    let no_index = "00:01:00,000 --> 00:01:20,000\nHello world !\n";
    let parsed = SubRipItem::from_string(no_index).unwrap();
    assert_eq!(parsed.index, ItemIndex::None);
    assert_eq!(parsed.text, "Hello world !");
}

#[test]
fn test_junk_after_timestamp() {
    let item = sample_item();
    let junk = "1\n00:01:00,000 --> 00:01:20,000?\nHello world !\n";
    let parsed = SubRipItem::from_string(junk).unwrap();
    assert_eq!(parsed, item);
}
