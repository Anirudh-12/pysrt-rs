use clap::{Parser, Subcommand};
use libsrt::{open, SubRipTime};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "srt", version, about = "A high-performance SubRip (.srt) subtitle editor (Port Mortem Track D)")]
struct Cli {
    /// Modify file in-place
    #[arg(short = 'i', long = "in-place", global = true)]
    in_place: bool,

    /// Character encoding of subtitle file
    #[arg(short = 'e', long = "encoding", global = true)]
    encoding: Option<String>,

    /// Output file path (default: stdout)
    #[arg(short = 'o', long = "output", global = true)]
    output: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Shift subtitle timestamps
    Shift {
        /// Time offset string, e.g. "1m12s", "500ms", "-1s500ms", "-3"
        offset: String,
        /// Input .srt subtitle file path
        file: PathBuf,
    },
    /// Convert timestamps between frame rates
    Rate {
        /// Current frame rate (e.g. 23.976 or 24)
        old_fps: f64,
        /// Target frame rate (e.g. 25)
        new_fps: f64,
        /// Input .srt subtitle file path
        file: PathBuf,
    },
    /// Split subtitle file into multiple parts
    Split {
        /// Timestamps to split at followed by input .srt subtitle file path, e.g. "48m18s" "movie.srt"
        #[arg(required = true, num_args = 2..)]
        args: Vec<String>,
    },
    /// Break long subtitle lines
    Break {
        /// Maximum number of characters per line
        length: usize,
        /// Input .srt subtitle file path
        file: PathBuf,
    },
}

fn parse_duration_str(s: &str) -> Result<i64, String> {
    let s_trim = s.trim();
    if s_trim.contains(':') {
        if let Ok(t) = SubRipTime::from_string(s_trim) {
            return Ok(t.ordinal);
        }
        let parts: Vec<&str> = s_trim.split([':', '.', ',']).collect();
        if parts.len() == 3 {
            if let (Ok(h), Ok(m), Ok(s_val)) = (
                parts[0].parse::<i64>(),
                parts[1].parse::<i64>(),
                parts[2].parse::<i64>(),
            ) {
                return Ok(h * 3600 * 1000 + m * 60 * 1000 + s_val * 1000);
            }
        }
    }

    let negative = s_trim.starts_with('-');
    let body = if negative { &s_trim[1..] } else { s_trim };

    let mut total_ms = 0i64;
    let mut i = 0;
    let chars: Vec<char> = body.chars().collect();

    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            return Err(format!("Expected digit at position {} in {}", i, s));
        }
        let start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        let num_str: String = chars[start..i].iter().collect();
        let num: i64 = num_str.parse().map_err(|_| format!("Invalid number in {}", s))?;

        let unit_start = i;
        while i < chars.len() && chars[i].is_ascii_alphabetic() {
            i += 1;
        }
        let unit: String = chars[unit_start..i].iter().collect();
        match unit.to_lowercase().as_str() {
            "h" => total_ms += num * 3600 * 1000,
            "m" => total_ms += num * 60 * 1000,
            "s" => total_ms += num * 1000,
            "ms" => total_ms += num,
            "" => total_ms += num * 1000, // Default unit is seconds
            _ => return Err(format!("Unknown time unit '{}' in {}", unit, s)),
        }
    }

    if negative {
        Ok(-total_ms)
    } else {
        Ok(total_ms)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Shift { offset, file } => {
            let total_ms = parse_duration_str(&offset)?;
            let mut srt = open(&file, cli.encoding.as_deref())?;
            srt.shift(0, 0, 0, total_ms, None);

            if cli.in_place {
                let backup = file.with_extension("srt.bak");
                fs::copy(&file, backup)?;
                srt.save(Some(&file))?;
            } else if let Some(out_path) = cli.output {
                srt.save(Some(out_path))?;
            } else {
                print!("{}", srt.text());
            }
        }
        Commands::Rate {
            old_fps,
            new_fps,
            file,
        } => {
            let ratio = new_fps / old_fps;
            let mut srt = open(&file, cli.encoding.as_deref())?;
            srt.shift(0, 0, 0, 0, Some(ratio));

            if cli.in_place {
                let backup = file.with_extension("srt.bak");
                fs::copy(&file, backup)?;
                srt.save(Some(&file))?;
            } else if let Some(out_path) = cli.output {
                srt.save(Some(out_path))?;
            } else {
                print!("{}", srt.text());
            }
        }
        Commands::Split { args } => {
            if args.len() < 2 {
                return Err("split command requires at least one limit and a filename".into());
            }
            let file_str = &args[args.len() - 1];
            let limit_strs = &args[..args.len() - 1];
            let file = PathBuf::from(file_str);

            let srt = open(&file, cli.encoding.as_deref())?;

            let mut limits = vec![0i64];
            for l_str in limit_strs {
                let ms = parse_duration_str(l_str)?;
                limits.push(ms);
            }
            let last_end = srt
                .items
                .last()
                .map(|item| item.end.ordinal + 1)
                .unwrap_or(1);
            let max_limit = *limits.last().unwrap_or(&0);
            limits.push(last_end.max(max_limit + 1));

            let stem = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let ext = file.extension().and_then(|s| s.to_str());
            let parent = file.parent().unwrap_or_else(|| Path::new(""));

            for (index, window) in limits.windows(2).enumerate() {
                let (start, end) = (window[0], window[1]);
                let file_name = if let Some(ext_str) = ext {
                    format!("{}.{}.{}", stem, index + 1, ext_str)
                } else {
                    format!("{}.{}", stem, index + 1)
                };
                let out_path = if parent.as_os_str().is_empty() {
                    PathBuf::from(file_name)
                } else {
                    parent.join(file_name)
                };

                let mut part_file = srt.slice_by_time(
                    Some(SubRipTime::from_ordinal(end)),
                    None,
                    None,
                    Some(SubRipTime::from_ordinal(start)),
                );
                part_file.shift(0, 0, 0, -start, None);
                part_file.clean_indexes();
                part_file.save(Some(&out_path))?;
            }
        }
        Commands::Break { length, file } => {
            let mut srt = open(&file, cli.encoding.as_deref())?;
            srt.break_lines(length);

            if cli.in_place {
                let backup = file.with_extension("srt.bak");
                fs::copy(&file, backup)?;
                srt.save(Some(&file))?;
            } else if let Some(out_path) = cli.output {
                srt.save(Some(out_path))?;
            } else {
                print!("{}", srt.text());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_parsing() {
        assert_eq!(parse_duration_str("1s").unwrap(), 1000);
        assert_eq!(parse_duration_str("-500ms").unwrap(), -500);
        assert_eq!(parse_duration_str("-3").unwrap(), -3000);
        assert_eq!(parse_duration_str("1m12s").unwrap(), 72000);
        assert_eq!(parse_duration_str("01:00:00,000").unwrap(), 3600000);
        assert_eq!(parse_duration_str("00:48:18").unwrap(), 2898000);
    }

    #[test]
    fn test_cli_split_parsing() {
        let cli = Cli::try_parse_from(["srt", "split", "48m18s", "movie.srt"]).unwrap();
        match cli.command {
            Commands::Split { args } => {
                assert_eq!(args, vec!["48m18s", "movie.srt"]);
            }
            _ => panic!("Expected Split command"),
        }
    }

    #[test]
    fn test_cli_break_parsing() {
        let cli = Cli::try_parse_from(["srt", "break", "30", "movie.srt"]).unwrap();
        match cli.command {
            Commands::Break { length, file } => {
                assert_eq!(length, 30);
                assert_eq!(file, PathBuf::from("movie.srt"));
            }
            _ => panic!("Expected Break command"),
        }
    }
}
