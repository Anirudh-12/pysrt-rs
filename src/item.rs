use std::cmp::Ordering;
use std::fmt;
use crate::error::{Result, SrtError};
use crate::time::SubRipTime;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ItemIndex {
    Int(i32),
    Str(String),
    None,
}

impl Default for ItemIndex {
    fn default() -> Self {
        ItemIndex::Int(0)
    }
}

impl ItemIndex {
    pub fn as_i32(&self) -> i32 {
        match self {
            ItemIndex::Int(n) => *n,
            _ => 0,
        }
    }
}

impl fmt::Display for ItemIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemIndex::Int(n) => write!(f, "{}", n),
            ItemIndex::Str(s) => write!(f, "{}", s),
            ItemIndex::None => write!(f, "None"),
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct SubRipItem {
    pub index: ItemIndex,
    pub start: SubRipTime,
    pub end: SubRipTime,
    pub text: String,
    pub position: String,
}

impl SubRipItem {
    pub fn new(
        index: ItemIndex,
        start: SubRipTime,
        end: SubRipTime,
        text: String,
        position: String,
    ) -> Self {
        Self {
            index,
            start,
            end,
            text,
            position,
        }
    }

    pub fn duration(&self) -> SubRipTime {
        SubRipTime::from_ordinal(self.end.ordinal - self.start.ordinal)
    }

    pub fn text_without_tags(&self) -> String {
        let mut result = String::with_capacity(self.text.len());
        let mut in_tag = false;
        for c in self.text.chars() {
            if c == '<' {
                in_tag = true;
            } else if c == '>' && in_tag {
                in_tag = false;
            } else if !in_tag {
                result.push(c);
            }
        }
        result
    }

    pub fn characters_per_second(&self) -> f64 {
        let duration_secs = self.duration().ordinal as f64 / 1000.0;
        if duration_secs == 0.0 {
            return 0.0;
        }
        let text = self.text_without_tags();
        let char_count = text.chars().filter(|c| *c != '\n' && *c != '\r').count();
        char_count as f64 / duration_secs
    }

    pub fn shift(
        &mut self,
        hours: i64,
        minutes: i64,
        seconds: i64,
        milliseconds: i64,
        ratio: Option<f64>,
    ) {
        self.start
            .shift(hours, minutes, seconds, milliseconds, ratio);
        self.end
            .shift(hours, minutes, seconds, milliseconds, ratio);
    }

    pub fn from_string(source: &str) -> Result<Self> {
        let lines: Vec<&str> = source.lines().collect();
        Self::from_lines(lines)
    }

    pub fn from_lines(lines: Vec<&str>) -> Result<Self> {
        if lines.len() < 2 {
            return Err(SrtError::InvalidItem(
                "SubRipItem requires at least 2 lines".into(),
            ));
        }
        let mut lines: Vec<&str> = lines.into_iter().map(|l| l.trim_end()).collect();
        let mut index = ItemIndex::None;
        if !lines[0].contains("-->") {
            let idx_str = lines.remove(0);
            if let Ok(val) = idx_str.parse::<i32>() {
                index = ItemIndex::Int(val);
            } else {
                index = ItemIndex::Str(idx_str.to_string());
            }
        }
        if lines.is_empty() {
            return Err(SrtError::InvalidItem("Missing timestamp line".into()));
        }
        let (start, end, position) = Self::split_timestamps(lines[0])?;
        let text = lines[1..].join("\n");
        Ok(Self {
            index,
            start,
            end,
            text,
            position,
        })
    }

    pub fn split_timestamps(line: &str) -> Result<(SubRipTime, SubRipTime, String)> {
        let parts: Vec<&str> = line.split("-->").collect();
        if parts.len() != 2 {
            return Err(SrtError::InvalidItem(format!(
                "Invalid timestamp line: {}",
                line
            )));
        }
        let start_str = parts[0].trim();
        let start = SubRipTime::from_string(start_str)?;

        let right = parts[1].trim_start();
        let (end_str, position) = match right.split_once(' ') {
            Some((end_part, pos_part)) => (end_part, pos_part.to_string()),
            None => (right, String::new()),
        };
        let end = SubRipTime::from_string(end_str)?;

        Ok((start, end, position))
    }
}

impl Ord for SubRipItem {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.start, self.end).cmp(&(other.start, other.end))
    }
}

impl PartialOrd for SubRipItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for SubRipItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pos_str = if self.position.trim().is_empty() {
            String::new()
        } else {
            format!(" {}", self.position)
        };
        write!(
            f,
            "{}\n{} --> {}{}\n{}\n",
            self.index, self.start, self.end, pos_str, self.text
        )
    }
}

impl fmt::Debug for SubRipItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SubRipItem(index={}, start={}, end={}, text={:?}, position={:?})",
            self.index, self.start, self.end, self.text, self.position
        )
    }
}
