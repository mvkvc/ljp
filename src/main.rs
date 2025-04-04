use anyhow::{Context, Result};
use clap::Parser;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    io::{self, stdin, Write},
    str::FromStr,
};
mod sets;

use sets::hiragana::HiraganaStudySet;
use sets::katakana::KatakanaStudySet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyItem {
    front: String,
    back: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudySession {
    sets: Vec<String>,
    items: Vec<StudyItem>,
    #[serde(default)]
    weights: Vec<u32>,
    #[serde(skip)]
    dist: Option<WeightedIndex<u32>>,
    #[serde(skip)]
    rng: ThreadRng,
}

impl StudySession {
    fn new(sets: Vec<String>) -> Result<Self> {
        let mut resolved_sets = Vec::new();
        let mut items = Vec::new();
        for set_name in sets {
            if let Some(resolved_set) = get_set(&set_name) {
                resolved_sets.push(resolved_set.name());
                items.extend(resolved_set.load());
            } else {
                eprintln!("Warning: Set '{}' not found.", set_name);
            }
        }

        let weights = vec![1; items.len()];
        let dist = if !weights.is_empty() {
            Some(
                WeightedIndex::new(&weights)
                    .context("Failed to create weighted index for study items")?,
            )
        } else {
            None
        };

        Ok(Self {
            sets: resolved_sets,
            items,
            weights,
            dist,
            rng: rand::rng(),
        })
    }

    fn sync_dist(&mut self) -> Result<()> {
        if !self.weights.is_empty() {
            self.dist =
                Some(WeightedIndex::new(&self.weights).context("Failed to sync weighted index")?);
        }
        Ok(())
    }

    fn increment(&mut self) -> Result<()> {
        self.weights.iter_mut().for_each(|w| *w += 1);
        self.sync_dist()?;
        Ok(())
    }

    fn reset(&mut self, index: usize) -> Result<()> {
        if index < self.weights.len() {
            self.weights[index] = 1;
            self.sync_dist()?;
        }
        Ok(())
    }

    fn sample(&mut self) -> Option<(usize, StudyItem)> {
        if let Some(dist) = self.dist.as_ref() {
            let index = dist.sample(&mut self.rng);
            self.items.get(index).map(|item| (index, item.clone()))
        } else {
            None
        }
    }
}

pub trait StudySetLoader {
    fn name(&self) -> String;
    fn load(&self) -> Vec<StudyItem>;
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "hiragana")]
    sets: String,
    #[arg(short, long, default_value = "false")]
    list: bool,
}

fn get_set(name: &str) -> Option<Box<dyn StudySetLoader>> {
    match name {
        "hiragana" => Some(Box::new(HiraganaStudySet)),
        "katakana" => Some(Box::new(KatakanaStudySet)),
        _ => None,
    }
}

enum Commands {
    Answer(String),
    Help,
    Weights,
    Quit,
}

impl Commands {
    fn help() {
        println!("Available commands:");
        println!("  \\h        - Show this help message");
        println!("  \\w        - Show weights for current items");
        println!("  \\q        - Quit the study session");
        println!("  <answer> - Enter your answer for the current item");
    }
}

impl FromStr for Commands {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "\\h" => Ok(Commands::Help),
            "\\w" => Ok(Commands::Weights),
            "\\q" => Ok(Commands::Quit),
            _ if s.starts_with('\\') => Err("Unknown command".to_string()),
            _ => Ok(Commands::Answer(s.to_string())),
        }
    }
}

fn run_session(session: &mut StudySession) -> Result<()> {
    loop {
        let (item_index, item) = match session.sample() {
            Some((idx, it)) => (idx, it),
            None => {
                println!("No items available for study. Exiting session.");
                break Ok(());
            }
        };

        println!("\n{}", item.front);
        print!("|> ");

        io::stdout().flush().context("Failed to flush stdout")?;

        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .context("Failed to read line from stdin")?;

        match Commands::from_str(input.trim()) {
            Ok(Commands::Help) => {
                Commands::help();
                continue;
            }
            Ok(Commands::Weights) => {
                let mut weighted_items: Vec<_> = session
                    .weights
                    .iter()
                    .zip(session.items.iter())
                    .map(|(&w, item)| (w, &item.front, &item.back))
                    .collect();

                weighted_items.sort_by(|a, b| b.0.cmp(&a.0));

                for (weight, front, back) in weighted_items {
                    println!("{} / {} / {:<3}", front, back, weight);
                }
                println!();
                continue;
            }
            Ok(Commands::Quit) => {
                println!("Quitting...");
                break Ok(());
            }
            Ok(Commands::Answer(answer)) => {
                if answer == item.back {
                    println!("Correct!");
                    session.reset(item_index)?;
                } else {
                    println!("Incorrect. The correct answer is: {}", item.back);
                }
            }
            Err(e) => {
                eprintln!("Invalid command: {}. Type \\q to quit.", e);
                continue;
            }
        }

        session.increment()?;
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.list {
        println!("Available sets: hiragana, katakana");
        return Ok(());
    }

    let set_names: Vec<String> = args.sets.split(',').map(String::from).collect();

    let mut session = StudySession::new(set_names)?;

    let mut display_sets = session.sets.clone();
    display_sets.sort();
    println!(
        "Starting session for {} items from sets: {}",
        session.items.len(),
        display_sets.join(", ")
    );
    println!("Type '\\h' for commands.");

    run_session(&mut session)?;

    Ok(())
}
