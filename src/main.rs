use anyhow::{anyhow, bail, Result};
use std::collections::{HashMap, HashSet};

use chrono::prelude::*;

fn print_help() {
    println!("Usage: chat [--word <word>] [--count] [--dist] [--limit <limit>]");
    println!();
    println!("--limit <limit> - Limit the number of users to show (default: 20).");
    println!("--word <word>   - Add a word to the list to search for. This will cause `words.png` to be written and print word usage statistics to console.");
    println!("--any           - Like `--word <word>`, but matches any words.");
    println!("--count         - Perform overall aggregation over total word count use.");
    println!("--dist          - Write a `contributions.png` which contains a distribution of the percentage of users contributing to chat.");
}

fn main() -> Result<()> {
    let mut words = HashSet::new();
    let mut count = false;
    let mut dist = false;
    let mut limit = 20;
    let mut any_word = false;

    let mut it = std::env::args();
    it.next();

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--word" => {
                let word = it
                    .next()
                    .ok_or_else(|| anyhow!("missing argument to `--word`"))?;

                words.insert(word.to_lowercase());
            }
            "--any" => {
                any_word = true;
            }
            "--count" => {
                count = true;
            }
            "--dist" => {
                dist = true;
            }
            "--limit" => {
                let l = it
                    .next()
                    .ok_or_else(|| anyhow!("missing argument to `--limit`"))?;
                limit =
                    str::parse::<usize>(&l).map_err(|_| anyhow!("bad argument to `--limit`"))?;
            }
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            arg => {
                print_help();
                bail!("Unsupported argument `{}`", arg);
            }
        }
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

                if any_word || words.contains(&w) {
                    *offenders.entry(record.author.clone()).or_default() += 1;
                    *months.entry(date.month() - 1).or_default() += 1;
                }
            }
        }
    }

    if !words.is_empty() || any_word {
        let title = if !any_word {
            let mut word = words
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

        format_list(title, &offenders, limit, "")?;
    }

    if count {
        format_list(format!("Top chatters"), &counts, limit, " words")?;
    }

    if dist {
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
