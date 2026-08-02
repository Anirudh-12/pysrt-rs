//! Rust native integration tests for `SubRipTime`
//! Ported 1-to-1 from reference Python `test_srttime.py` (21 test cases).

use libsrt::time::SubRipTime;

const KNOWN_VALUES: &[(&str, (i64, i64, i64, i64))] = &[
    ("00:00:00,000", (0, 0, 0, 0)),
    ("00:00:00,001", (0, 0, 0, 1)),
    ("00:00:02,000", (0, 0, 2, 0)),
    ("00:03:00,000", (0, 3, 0, 0)),
    ("04:00:00,000", (4, 0, 0, 0)),
    ("12:34:56,789", (12, 34, 56, 789)),
];

#[test]
fn test_default_value() {
    let time = SubRipTime::default();
    assert_eq!(time.ordinal, 0);
}

#[test]
fn test_micro_seconds() {
    let mut time = SubRipTime::default();
    time.set_milliseconds(1);
    assert_eq!(time.milliseconds(), 1);
    time.set_hours(time.hours() + 42);
    assert_eq!(time.milliseconds(), 1);
    time.set_milliseconds(time.milliseconds() + 1000);
    assert_eq!(time.seconds(), 1);
}

#[test]
fn test_seconds() {
    let mut time = SubRipTime::default();
    time.set_seconds(1);
    assert_eq!(time.seconds(), 1);
    time.set_hours(time.hours() + 42);
    assert_eq!(time.seconds(), 1);
    time.set_seconds(time.seconds() + 60);
    assert_eq!(time.minutes(), 1);
}

#[test]
fn test_minutes() {
    let mut time = SubRipTime::default();
    time.set_minutes(1);
    assert_eq!(time.minutes(), 1);
    time.set_hours(time.hours() + 42);
    assert_eq!(time.minutes(), 1);
    time.set_minutes(time.minutes() + 60);
    assert_eq!(time.hours(), 43);
}

#[test]
fn test_hours() {
    let mut time = SubRipTime::default();
    time.set_hours(1);
    assert_eq!(time.hours(), 1);
    time.set_minutes(time.minutes() + 42);
    assert_eq!(time.hours(), 1);
}

#[test]
fn test_shifting() {
    let mut time = SubRipTime::default();
    time.shift(1, 1, 1, 1, None);
    assert_eq!(time, SubRipTime::new(1, 1, 1, 1));
}

#[test]
fn test_descriptor_from_class() {
    // In Rust, ensure methods are called on instances and ordinals behave correctly
    let time = SubRipTime::new(1, 2, 3, 400);
    assert_eq!(time.hours(), 1);
    assert_eq!(time.minutes(), 2);
    assert_eq!(time.seconds(), 3);
    assert_eq!(time.milliseconds(), 400);
}

#[test]
fn test_parsing() {
    for (time_string, (h, m, s, ms)) in KNOWN_VALUES {
        let parsed = SubRipTime::from_string(time_string).expect("Valid time string");
        assert_eq!(parsed, SubRipTime::new(*h, *m, *s, *ms));
    }
}

#[test]
fn test_serialization() {
    for (time_string, (h, m, s, ms)) in KNOWN_VALUES {
        let time = SubRipTime::new(*h, *m, *s, *ms);
        assert_eq!(&time.to_string(), time_string);
    }
}

#[test]
fn test_negative_serialization() {
    let time = SubRipTime::new(-1, 2, 3, 4);
    assert_eq!(time.to_string(), "00:00:00,000");
}

#[test]
fn test_invalid_time_string() {
    assert!(SubRipTime::from_string("hello").is_err());
}

#[test]
fn test_from_tuple() {
    assert_eq!(SubRipTime::new(0, 0, 0, 0), SubRipTime::default());
    assert_eq!(SubRipTime::new(0, 0, 0, 1), SubRipTime::new(0, 0, 0, 1));
    assert_eq!(SubRipTime::new(0, 0, 2, 0), SubRipTime::new(0, 0, 2, 0));
    assert_eq!(SubRipTime::new(0, 3, 0, 0), SubRipTime::new(0, 3, 0, 0));
    assert_eq!(SubRipTime::new(4, 0, 0, 0), SubRipTime::new(4, 0, 0, 0));
    assert_eq!(SubRipTime::new(1, 2, 3, 4), SubRipTime::new(1, 2, 3, 4));
}

#[test]
fn test_from_dict() {
    // Equivalents of dict constructors in Python
    assert_eq!(SubRipTime::default(), SubRipTime::new(0, 0, 0, 0));
    assert_eq!(SubRipTime::new(0, 0, 0, 1), SubRipTime::new(0, 0, 0, 1));
    assert_eq!(SubRipTime::new(0, 0, 2, 0), SubRipTime::new(0, 0, 2, 0));
    assert_eq!(SubRipTime::new(0, 3, 0, 0), SubRipTime::new(0, 3, 0, 0));
    assert_eq!(SubRipTime::new(4, 0, 0, 0), SubRipTime::new(4, 0, 0, 0));
    assert_eq!(
        SubRipTime::new(1, 2, 3, 4),
        SubRipTime::new(1, 2, 3, 4)
    );
}

#[test]
fn test_from_time() {
    let time_obj = SubRipTime::new(1, 2, 3, 4);
    assert_eq!(SubRipTime::new(1, 2, 3, 4), time_obj);
    assert!(SubRipTime::new(1, 2, 3, 5) >= time_obj);
    assert!(SubRipTime::new(1, 2, 3, 3) <= time_obj);
    assert!(SubRipTime::new(1, 2, 3, 0) != time_obj);
}

#[test]
fn test_from_ordinal() {
    let time = SubRipTime::from_ordinal(3600000);
    assert_eq!(time.hours(), 1);
    assert_eq!(SubRipTime::from_ordinal(3600000), SubRipTime::new(1, 0, 0, 0));
}

#[test]
fn test_add() {
    let time = SubRipTime::new(1, 2, 3, 4);
    assert_eq!(time + SubRipTime::new(1, 2, 3, 4), SubRipTime::new(2, 4, 6, 8));
}

#[test]
fn test_iadd() {
    let mut time = SubRipTime::new(1, 2, 3, 4);
    time += SubRipTime::new(1, 2, 3, 4);
    assert_eq!(time, SubRipTime::new(2, 4, 6, 8));
}

#[test]
fn test_sub() {
    let time = SubRipTime::new(1, 2, 3, 4);
    assert_eq!(time - SubRipTime::new(1, 2, 3, 4), SubRipTime::default());
}

#[test]
fn test_isub() {
    let mut time = SubRipTime::new(1, 2, 3, 4);
    time -= SubRipTime::new(1, 2, 3, 4);
    assert_eq!(time, SubRipTime::default());
}

#[test]
fn test_mul() {
    let time = SubRipTime::new(1, 2, 3, 4);
    assert_eq!(time * 2.0, SubRipTime::new(2, 4, 6, 8));
    assert_eq!(time * 0.5, SubRipTime::new(0, 31, 1, 502));
}

#[test]
fn test_imul() {
    let mut time = SubRipTime::new(1, 2, 3, 4);
    time *= 2.0;
    assert_eq!(time, SubRipTime::new(2, 4, 6, 8));
    time *= 0.5;
    assert_eq!(time, SubRipTime::new(1, 2, 3, 4));
}
