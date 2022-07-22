use std::process::id;
use std::{collections::HashMap, path::PathBuf};
use std::{fs, io};

use clap::Parser;
use grep::matcher::LineTerminator;
use grep::regex::RegexMatcherBuilder;
use grep::searcher::{sinks, SearcherBuilder, Sink};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use pg_query::ast::Node;
use regex::Regex;

use crate::parser::SchemaParser;

mod objects;
mod parser;
mod statements;
mod typedvec;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, parse(from_os_str), default_value = "schema/")]
    path: PathBuf,
}


fn main() {
    let args: Cli = Cli::parse();
    let path = args.path;
    println!("Searching path: {:?}", path);
    if !path.exists() {
        println!("Path {:?} does not exist!", path);
        std::process::exit(1);
    }

    // IMPORTANT: use `\s+` instead of simple spaces, the SQL parser discards
    // all whitespace characters
    let entity_pattern: String = [
        "database",
        "domain",
        r"foreign\s+data\s+wrapper",
        r"foreign\s+table",
        "function",
        "index",
        r"materialized\s+view",
        "role",
        "schema",
        "server",
        "table",
        "type",
        "view", // CREATE TABLE AS puts the table name in the same place as regular CREATE TABLE syntax
    ]
    .join("|")
    .to_string();

    // PostgreSQL identifiers start with a letter or underscore,
    // and can only contain letters, numbers, underscores or dollar signs $
    let identifier = "[a-zA-Z_][a-zA-Z0-9_$]*";
    // Add optional quotes, without capturing them
    // This handling is not robust (allows only one of the two quotes), but simple
    // The real PostgreSQL parser will signal errors
    let quoted_identifier = format!(r#""?({})"?"#, identifier);
    let pattern = format!(
        r"create\s+(?:{})\s+{}(?:\.{})?",
        entity_pattern, quoted_identifier, quoted_identifier
    );
    let regex = Regex::new(&pattern).unwrap();

    println!("{}", quoted_identifier);
    println!("{}", pattern);

    let glob = OverrideBuilder::new(path.clone())
        .add("**/*.sql")
        .unwrap()
        .build()
        .unwrap();
    let walker = WalkBuilder::new(path).overrides(glob).build();

    let mut parser = SchemaParser::new();

    for file in walker.into_iter() {
        let file = file.unwrap();
        if file.file_type().unwrap().is_dir() {
            continue;
        }
        parser.parse(file);
    }

    // let paths = fs::read_dir(pathArg).unwrap();
    // for path in paths {
    //     println!("Name: {}", path.unwrap().path().display())
    // }
}
