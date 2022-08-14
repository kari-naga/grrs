use clap::Parser;
use std::fs::File;
use std::io::{stdout, BufWriter, Write, BufReader, BufRead};
use anyhow::{Context, Result};
use terminal_size::{Width, terminal_size};
use ansi_term::Colour::Red;
use regex::Regex;

#[derive(Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
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

fn main() -> Result<()> {
    let args = Args::parse();

    let mut handle = BufWriter::new(stdout().lock());

    let file = File::open(&args.path)
        .with_context(|| format!("could not open file `{}`", &args.path.display()))?;

    let lines = BufReader::new(file).lines().enumerate();

    let re = Regex::new(&[
        if args.case_insensitive {"(?i)"} else {""},
        if args.match_words {r"\b(?P<match>"} else {"(?P<match>"},
        &args.pattern,
        if args.match_words {r")\b"} else {")"},
    ].concat()).with_context(|| format!("could not compile regex for pattern `{}`", &args.pattern))?;

    let mut in_terminal = true;

    let width = match terminal_size() {
        Some((Width(width), _)) => width as usize,
        None => {
            in_terminal = false;
            120
        },
    };

    let separator = "-".repeat(width);

    let mut last_idx = 0;

    let mut count = 0;
    
    for (idx, line) in lines {
        let idx = idx + 1;
        let line = line.with_context(|| format!("could not read line {idx} in file `{}`", &args.path.display()))?;
        let match_result = re.is_match(&line);
        if if args.invert_match {!match_result} else {match_result} {
            if last_idx != 0 && idx - last_idx > 1 {
                writeln!(handle, "{separator}")
                    .with_context(|| "could not print separator")?;
            }
            let end = format!("line {}", idx);
            let max_len = width - end.len() - 3;
            let line = if line.len() > max_len {
                (&line[..max_len - 3]).to_string() + "..."
            } else {
                line
            };
            let len = line.len();
            let line = if in_terminal && !args.invert_match && args.highlight_matches {
                re.replace_all(&line, Red.bold().paint("$match").to_string()).to_string()
            } else {
                line
            };
            let align = width - len;
            writeln!(handle, "{line}{end:>align$}")
                .with_context(|| format!("could not print line {idx} in file `{}`", &args.path.display()))?;
            last_idx = idx;
            count += 1;
        }
    }

    if count == 0 {
        writeln!(handle, "found no matches")
            .with_context(|| "could not print message")?;
    } else if args.count_matches {
        writeln!(handle, "{separator}")
            .with_context(|| "could not print separator")?;
        writeln!(handle, "found {count} match{}", if count > 1 {"es"} else {""})
            .with_context(|| "could not print message")?;
    }

    Ok(())
}
