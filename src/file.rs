use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::ops::{Deref, DerefMut};
use encoding_rs::Encoding;
use crate::error::{Result, SrtError};
use crate::item::{ItemIndex, SubRipItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorHandling {
    Pass = 0,
    Log = 1,
    Raise = 2,
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct SubRipFile {
    pub items: Vec<SubRipItem>,
    pub eol: String,
    pub path: Option<String>,
    pub encoding: String,
}

impl SubRipFile {
    pub fn new(
        items: Vec<SubRipItem>,
        eol: Option<String>,
        path: Option<String>,
        encoding: Option<String>,
    ) -> Self {
        Self {
            items,
            eol: eol.unwrap_or_else(|| "\n".to_string()),
            path,
            encoding: encoding.unwrap_or_else(|| "utf-8".to_string()),
        }
    }

    pub fn guess_eol(source: &str) -> String {
        if source.contains("\r\n") {
            "\r\n".to_string()
        } else if source.contains('\r') && !source.contains('\n') {
            "\r".to_string()
        } else {
            "\n".to_string()
        }
    }

    pub fn from_string(source: &str) -> Result<Self> {
        Self::from_string_with_error_handling(source, ErrorHandling::Raise)
    }

    pub fn from_string_with_error_handling(
        source: &str,
        error_handling: ErrorHandling,
    ) -> Result<Self> {
        let eol = Self::guess_eol(source);
        let items = Self::parse_str(source, error_handling)?;
        Ok(Self::new(items, Some(eol), None, None))
    }

    pub fn parse_str(source: &str, error_handling: ErrorHandling) -> Result<Vec<SubRipItem>> {
        let mut items = Vec::new();
        let mut buffer = Vec::new();

        for line in source.lines() {
            if !line.trim().is_empty() {
                buffer.push(line);
            } else if !buffer.is_empty() {
                match SubRipItem::from_lines(buffer.clone()) {
                    Ok(item) => items.push(item),
                    Err(e) => match error_handling {
                        ErrorHandling::Raise => return Err(e),
                        ErrorHandling::Log => {
                            eprintln!("Warning: Skipping invalid item: {}", e);
                        }
                        ErrorHandling::Pass => {}
                    },
                }
                buffer.clear();
            }
        }

        if !buffer.is_empty() {
            match SubRipItem::from_lines(buffer) {
                Ok(item) => items.push(item),
                Err(e) => match error_handling {
                    ErrorHandling::Raise => return Err(e),
                    ErrorHandling::Log => {
                        eprintln!("Warning: Skipping invalid item: {}", e);
                    }
                    ErrorHandling::Pass => {}
                },
            }
        }

        Ok(items)
    }

    pub fn open<P: AsRef<Path>>(path: P, encoding_name: Option<&str>) -> Result<Self> {
        let path_ref = path.as_ref();
        if path_ref.to_string_lossy() == "/dev/null" {
            return Ok(Self::default());
        }

        let mut file = File::open(path_ref)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let (decoded, actual_encoding) = if let Some(enc_str) = encoding_name {
            if enc_str.eq_ignore_ascii_case("utf-32-le") || enc_str.eq_ignore_ascii_case("utf_32_le") {
                let s = Self::decode_utf32_le(&bytes)?;
                (s, "utf_32_le".to_string())
            } else if enc_str.eq_ignore_ascii_case("utf-32-be") || enc_str.eq_ignore_ascii_case("utf_32_be") {
                let s = Self::decode_utf32_be(&bytes)?;
                (s, "utf_32_be".to_string())
            } else if let Some(enc) = Encoding::for_label(enc_str.as_bytes()) {
                let (dec, _, _) = enc.decode(&bytes);
                let name = match enc.name() {
                    "UTF-8" => "utf_8",
                    "windows-1252" => "cp1252",
                    other => other,
                };
                (dec.into_owned(), name.to_string())
            } else {
                return Err(SrtError::Encoding(format!("Unsupported encoding: {}", enc_str)));
            }
        } else {
            if bytes.len() >= 4 && bytes[0..4] == [0xFF, 0xFE, 0x00, 0x00] {
                let s = Self::decode_utf32_le(&bytes[4..])?;
                (s, "utf_32_le".to_string())
            } else if bytes.len() >= 4 && bytes[0..4] == [0x00, 0x00, 0xFE, 0xFF] {
                let s = Self::decode_utf32_be(&bytes[4..])?;
                (s, "utf_32_be".to_string())
            } else {
                let (dec, enc, _) = encoding_rs::UTF_8.decode(&bytes);
                let name = match enc.name() {
                    "UTF-8" => "utf_8",
                    "windows-1252" => "cp1252",
                    other => other,
                };
                (dec.into_owned(), name.to_string())
            }
        };

        let mut srt_file = Self::from_string(&decoded)?;
        srt_file.path = Some(path_ref.to_string_lossy().to_string());
        srt_file.encoding = actual_encoding;
        Ok(srt_file)
    }

    fn decode_utf32_le(bytes: &[u8]) -> Result<String> {
        let mut chars = Vec::new();
        for chunk in bytes.chunks_exact(4) {
            let u = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            if let Some(c) = std::char::from_u32(u) {
                chars.push(c);
            }
        }
        Ok(chars.into_iter().collect())
    }

    fn decode_utf32_be(bytes: &[u8]) -> Result<String> {
        let mut chars = Vec::new();
        for chunk in bytes.chunks_exact(4) {
            let u = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            if let Some(c) = std::char::from_u32(u) {
                chars.push(c);
            }
        }
        Ok(chars.into_iter().collect())
    }

    pub fn save<P: AsRef<Path>>(&self, path: Option<P>) -> Result<()> {
        let target_path = path
            .map(|p| p.as_ref().to_string_lossy().to_string())
            .or_else(|| self.path.clone())
            .ok_or_else(|| {
                SrtError::Io(std::io::Error::other(
                    "No file path specified for save",
                ))
            })?;

        let mut file = File::create(target_path)?;
        self.write_into(&mut file)?;
        Ok(())
    }

    pub fn shift(
        &mut self,
        hours: i64,
        minutes: i64,
        seconds: i64,
        milliseconds: i64,
        ratio: Option<f64>,
    ) {
        for item in self.items.iter_mut() {
            item.shift(hours, minutes, seconds, milliseconds, ratio);
        }
    }

    pub fn text(&self) -> String {
        let mut buf = String::new();
        for (i, item) in self.items.iter().enumerate() {
            buf.push_str(&item.to_string());
            if i + 1 < self.items.len() {
                buf.push_str(&self.eol);
            }
        }
        buf
    }

    pub fn write_into<W: Write>(&self, writer: &mut W) -> Result<()> {
        for (i, item) in self.items.iter().enumerate() {
            write!(writer, "{}", item)?;
            if i + 1 < self.items.len() {
                write!(writer, "{}", self.eol)?;
            }
        }
        Ok(())
    }

    pub fn sort(&mut self) {
        self.items.sort();
    }

    pub fn clean_indexes(&mut self) {
        self.sort();
        for (i, item) in self.items.iter_mut().enumerate() {
            item.index = ItemIndex::Int((i + 1) as i32);
        }
    }

    pub fn slice(&self, start: usize, end: usize) -> Vec<SubRipItem> {
        let len = self.items.len();
        let s = start.min(len);
        let e = end.min(len).max(s);
        self.items[s..e].to_vec()
    }

    pub fn at(&self, time: crate::time::SubRipTime) -> Vec<&SubRipItem> {
        self.items
            .iter()
            .filter(|item| item.start <= time && item.end >= time)
            .collect()
    }
}

impl Deref for SubRipFile {
    type Target = [SubRipItem];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl DerefMut for SubRipFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}
