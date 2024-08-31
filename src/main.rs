use std::{
    fs::{metadata, File},
    io::{stdin, stdout, Write},
    process::exit,
};

use anyhow::Result;
use clap::{Arg, ArgAction, Command};
use parser::Parser;

mod model;
mod parser;

const DEFAULT_ENV_SAMPLE: &str = ".env.sample";
const DEFAULT_ENV: &str = ".env";
const DEFAULT_RANDOM_CHARS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*(-_=+)";

fn should_write_to(path: &str) -> Result<bool> {
    if metadata(path).is_ok() {
        print!(
            "{} already exists. Do you want to overwrite it? (y/n) ",
            path
        );
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        let input = input.trim();
        match input.to_lowercase().as_str() {
            "y" | "yes" => {
                return Ok(true);
                // Perform the overwrite operation here
            }
            "n" | "no" => {
                return Ok(false);
            }
            _ => return should_write_to(path),
        }
    }
    Ok(true)
}

fn main() -> Result<()> {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            clap::Arg::new("target")
                .long("target")
                .short('t')
                .default_value(DEFAULT_ENV)
                .help("File to write the result"),
        )
        .arg(
            Arg::new("source")
                .long("source")
                .short('s')
                .default_value(DEFAULT_ENV_SAMPLE)
                .help("File to use as a sample"),
        )
        .arg(
            Arg::new("overwrite")
                .long("overwrite")
                .short('o')
                .action(ArgAction::SetTrue)
                .help("Overwrites target file without asking for user input"),
        )
        .arg(
            Arg::new("use-default")
                .long("use-default")
                .short('u')
                .action(ArgAction::SetTrue)
                .help("Use default values without asking for user input"),
        )
        .arg(
            Arg::new("chars-for-random-string")
                .long("chars-for-random-string")
                .short('c')
                .default_value(DEFAULT_RANDOM_CHARS)
                .help("Characters used to create random strings"),
        )
        .get_matches();

    let target = matches.get_one::<String>("target").unwrap();
    let overwrite = matches.get_one::<bool>("overwrite").unwrap();
    if !overwrite && !should_write_to(target)? {
        exit(0);
    }

    let source = matches.get_one::<String>("source").unwrap();
    let use_default = matches.get_one::<bool>("use-default").unwrap();
    let chars = matches
        .get_one::<String>("chars-for-random-string")
        .unwrap();

    let mut parser = Parser::new(source.as_str(), chars, use_default)?;
    parser.parse(&mut stdin().lock())?;

    let mut output = File::create(target)?;
    output.write_all(parser.to_string().as_bytes())?;
    Ok(())
}
