use proptest::prelude::*;
use pysrt::{SubRipFile, SubRipItem, SubRipTime};

proptest! {
    #[test]
    fn prop_time_roundtrip(
        hours in 0..99i64,
        minutes in 0..59i64,
        seconds in 0..59i64,
        milliseconds in 0..999i64,
    ) {
        let t = SubRipTime::new(hours, minutes, seconds, milliseconds);
        let s = t.to_string();
        let parsed = SubRipTime::from_string(&s).expect("valid time string should parse");
        prop_assert_eq!(t, parsed);
    }

    #[test]
    fn prop_shift_identity(
        hours in 0..10i64,
        minutes in 0..59i64,
        seconds in 0..59i64,
        milliseconds in 0..999i64,
        shift_ms in -1000..1000i64,
    ) {
        let mut t1 = SubRipTime::new(hours, minutes, seconds, milliseconds);
        let orig = t1;
        t1.shift(0, 0, 0, shift_ms, None);
        t1.shift(0, 0, 0, -shift_ms, None);
        prop_assert_eq!(t1, orig);
    }

    #[test]
    fn prop_item_roundtrip(
        index in 1..1000i32,
        s_s in 0..3600i64,
        s_ms in 0..999i64,
        dur_s in 1..30i64,
        dur_ms in 0..999i64,
        text in "\\PC*",
    ) {
        let start = SubRipTime::new(0, 0, s_s, s_ms);
        let end = SubRipTime::new(0, 0, s_s + dur_s, dur_ms);
        let item = SubRipItem::new(index, start, end, text.clone(), String::new());
        let s = item.to_string();
        if let Ok(parsed) = SubRipItem::from_string(&s) {
            prop_assert_eq!(parsed.index, index);
            prop_assert_eq!(parsed.start, start);
            prop_assert_eq!(parsed.end, end);
        }
    }

    #[test]
    fn prop_file_sorting_invariant(
        indices in prop::collection::vec(1..100i32, 1..20),
        starts in prop::collection::vec(0..10000i64, 1..20),
    ) {
        let items: Vec<SubRipItem> = indices.iter().zip(starts.iter()).map(|(&idx, &st)| {
            SubRipItem::new(
                idx,
                SubRipTime::from_ordinal(st),
                SubRipTime::from_ordinal(st + 100),
                "Subtitle text".to_string(),
                String::new(),
            )
        }).collect();

        let mut srt_file = SubRipFile::new(items, None, None, None);
        srt_file.clean_indexes();

        for i in 1..srt_file.len() {
            prop_assert!(srt_file[i - 1].start <= srt_file[i].start);
            prop_assert_eq!(srt_file[i].index, (i + 1) as i32);
        }
    }
}

fn main() {
    println!("Run `cargo test --manifest-path fuzz/Cargo.toml` to execute the property-based fuzz tests.");
}
