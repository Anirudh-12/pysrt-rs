use clap::{Parser, Subcommand};
use pysrt::open;
use std::fs;
use std::path::PathBuf;

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
}

fn parse_duration_str(s: &str) -> Result<i64, String> {
    let s_trim = s.trim();
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
    }
}
