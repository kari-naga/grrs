use clap::Parser;
use std::fs::File;
use std::io::{stdout, BufWriter, Write, BufReader, BufRead};
use anyhow::{Context, Result};
use terminal_size::{terminal_size};
use ansi_term::Colour::Red;

#[derive(Parser)]
#[clap(
    version = "0.1.1",
    about = "A simple program to search files for a given pattern",
)]
struct Args {
    #[clap(help = "The pattern to search for")]
    pattern: String,
    
    #[clap(value_parser, help = "The file to search")]
    path: std::path::PathBuf,
    
    #[clap(short = 'i', long, action, help = "Ignore case")]
    case_insensitive: bool,

    #[clap(short = 'w', long, action, help = "Match full words only")]
    match_words: bool,

    #[clap(short = 'h', long, action, help = "Highlight matched patterns")]
    highlight_matches: bool,

    #[clap(short = 'v', long, action, help = "Return lines that do not match the pattern")]
    invert_match: bool,

    #[clap(short = 'c', long, action, help = "Count and output the number of matches")]
    count_matches: bool,
}

fn matches_pattern(line: &str, args: &Args) -> bool {
    match (args.case_insensitive, args.match_words) {
        (true, true) => line.split_whitespace().any(|s| s.to_lowercase() == args.pattern.to_lowercase()),
        (_, true) => line.split_whitespace().any(|s| s == args.pattern),
        (true, _) => line.to_lowercase().contains(&args.pattern.to_lowercase()),
        (_, _) => line.contains(&args.pattern)
    }
}

fn format(line: &String, args: &Args) -> String {
    match (args.case_insensitive, args.match_words) {
        (true, true) => line.split(" ")
            .map(|s| if s.to_lowercase() == args.pattern.to_lowercase() {Red.bold().paint(s).to_string()} else {s.to_string()})
            .collect::<Vec<_>>()
            .join(" "),
        (_, true) => line.split(" ")
            .map(|s| if s == args.pattern {Red.bold().paint(s).to_string()} else {s.to_string()})
            .collect::<Vec<_>>()
            .join(" "),
        (true, _) => {
            let mut modified_line = line.clone();
            let mut old_len = modified_line.len();
            for (idx, _) in line.to_lowercase().match_indices(&args.pattern.to_lowercase()) {
                let idx = idx + modified_line.len() - old_len;
                let end = idx + args.pattern.len();
                old_len = modified_line.len();
                modified_line.replace_range(idx..end, &Red.bold().paint(&modified_line[idx..end]).to_string());
            }
            modified_line
        },
        (_, _) => line.replace(&args.pattern, &Red.bold().paint(&args.pattern).to_string())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut handle = BufWriter::new(stdout().lock());

    let file = File::open(&args.path)
        .with_context(|| format!("could not open file `{}`", &args.path.display()))?;

    let (width, _) = terminal_size().with_context(|| "could not get terminal size")?;

    let lines = BufReader::new(file).lines().enumerate();

    let mut last_idx = 0;

    let mut count = 0;
    
    for (idx, line) in lines {
        let idx = idx + 1;
        let line = line.with_context(|| format!("could not read line {idx} in file `{}`", &args.path.display()))?;
        let match_result = matches_pattern(&line, &args);
        if if args.invert_match {!match_result} else {match_result} {
            if last_idx != 0 && idx - last_idx > 1 {
                writeln!(handle, "{}", "-".repeat(width.0 as usize))
                    .with_context(|| "could not print separator")?;
            }
            let end = format!("line {}", idx);
            let line = if line.len() > width.0 as usize - end.len() - 3 {(&line[..width.0 as usize - end.len() - 6]).to_string() + "..."} else {line};
            let len = line.len();
            let line = if !args.invert_match && args.highlight_matches {format(&line, &args)} else {line};
            writeln!(handle, "{line}{}{end}", " ".repeat(width.0 as usize - len - end.len()))
                .with_context(|| format!("could not print line {idx} in file `{}`", &args.path.display()))?;
            last_idx = idx;
            count += 1;
        }
    }

    if count == 0 {
        writeln!(handle, "found no matches")
            .with_context(|| "could not print message")?;
    } else if args.count_matches {
        writeln!(handle, "{}", "-".repeat(width.0 as usize))
            .with_context(|| "could not print separator")?;
        writeln!(handle, "found {count} match{}", if count > 1 {"es"} else {""})
            .with_context(|| "could not print message")?;
    }

    Ok(())
}
