use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};

use chrono::prelude::*;

fn main() -> Result<()> {
    let args = argwerk::args! {
        "chat [--word <word>] [--count] [--dist] [--limit <n>]" {
            words: HashSet<String> = HashSet::new(),
            count: bool = false,
            dist: bool = false,
            limit: usize = 20,
            any_word: bool = false,
            help: bool,
        }
        /// Add a word to the list to search for. This will cause `words.png` to
        /// be written and print word usage statistics to console.
        ["--word", word] => {
            words.insert(word.to_lowercase());
        }
        /// Like `--word <word>`, but matches any words.
        ["--any"] => {
            any_word = true;
        }
        /// Perform overall aggregation over total word count use.
        ["--count"] => {
            count = true;
        }
        /// Write a `contributions.png` which contains the distribution of the percentage of users contributing to chat.
        ["--dist"] => {
            dist = true;
        }
        /// Limit the number of users to show (default: 20).
        ["--limit", n] => {
            limit = str::parse::<usize>(&n)?;
        }
        ["-h" | "--help"] => {
            help = true;
        }
    }?;

    if args.help {
        println!("{}", args.help());
        return Ok(());
    }

    let root = std::env::current_dir()?;
    let exports = root.join("exports");

    if !exports.is_dir() {
        bail!("Missing directory: {}", exports.display());
    }

    let words_png = root.join("words.png");
    let contributions_png = root.join("contributions.png");

    let mut months = HashMap::<u32, u64>::new();
    let mut offenders = HashMap::<Box<str>, u64>::new();
    let mut counts = HashMap::<Box<str>, u64>::new();
    let mut total_words = 0;

    for (_, records) in chat::read_chats(&exports)? {
        for (n, record) in records.iter().enumerate() {
            let date = match NaiveDate::parse_from_str(&record.date, "%d-%b-%y %I:%M %p") {
                Ok(date) => date,
                Err(e) => {
                    dbg!(n, e, &record.date);
                    break;
                }
            };

            let date = Local.from_local_date(&date).unwrap();
            let count = counts.entry(record.author.clone()).or_default();

            for w in record.content.split(|c: char| !c.is_alphabetic()) {
                let w = w.trim_matches(|c: char| !c.is_alphabetic()).to_lowercase();

                if w.is_empty() {
                    continue;
                }

                total_words += 1;
                *count += 1;

                if args.any_word || args.words.contains(&w) {
                    *offenders.entry(record.author.clone()).or_default() += 1;
                    *months.entry(date.month() - 1).or_default() += 1;
                }
            }
        }
    }

    if !args.words.is_empty() || args.any_word {
        let title = if !args.any_word {
            let mut word = args
                .words
                .iter()
                .map(|s| format!("\"{}\"", s.as_str()))
                .collect::<Vec<_>>();
            word.sort();
            format!("Uses of {} by month", word.join(" / "))
        } else {
            String::from("Words by month")
        };

        let months = sorted_source(&months);
        chat::month_plot(&words_png, months, &title)?;
        println!("Wrote: {}", words_png.display());

        format_list(title, &offenders, args.limit, "")?;
    }

    if args.count {
        format_list(format!("Top chatters"), &counts, args.limit, " words")?;
    }

    if args.dist {
        let counts = sorted_source(&counts);
        let total = total_words as f64;
        let counts = counts
            .into_iter()
            .map(|(_, n)| n as f64 / total)
            .collect::<Vec<_>>();

        chat::contributions_per_user(&contributions_png, counts)?;
        println!("Wrote: {}", contributions_png.display());
    }

    Ok(())
}

fn sorted_source<K>(source: &HashMap<K, u64>) -> Vec<(K, u64)>
where
    K: Clone,
{
    let mut source = source
        .into_iter()
        .map(|(author, uses)| (author.clone(), *uses))
        .collect::<Vec<_>>();

    source.sort_by_key(|e| e.1);
    source
}

fn format_list(title: String, source: &HashMap<Box<str>, u64>, n: usize, what: &str) -> Result<()> {
    let source = sorted_source(source);

    println!("== {} ==", title);

    for (n, (author, count)) in source.into_iter().rev().take(n).enumerate() {
        println!("#{:02}: {:30} w/ {}{}", n + 1, author, count, what);
    }

    Ok(())
}
