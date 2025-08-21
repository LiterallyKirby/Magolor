
use std::env;
use std::fs;
use std::process::Command;
use anyhow::Result;
use logos::Logos;



mod modules {
    pub mod tokenizer;
    pub mod parser;
    pub mod IR;
}

fn main() -> Result<()> {
    // Grab command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <source file>", args[0]);
        return Ok(());
    }

    let filename = &args[1];

    // Read the source file
    println!("{}",filename);
    let source = fs::read_to_string(filename)?;

    let tokens = modules::tokenizer::tokenizeFile(&source);

    let AST = modules::parser::parseTokens(&tokens);

    println!("{:?}", AST);

    modules::IR::compile();

    Ok(())
}

