//! Basic usage example demonstrating no_std compatible code
//! This example shows how the parser can be used in no_std contexts

#![allow(dead_code)]

extern crate parser_rs;

use parser_rs::{Args, AtError, AtResult};

// Example function using Args in no_std
fn parse_args_example() -> AtResult<'static> {
    let args = Args { raw: "foo,bar,baz" };
    match args.get(1) {
        Some(val) => Ok(val),
        None => Err(AtError::InvalidArgs),
    }
}

// Example of error handling
fn handle_error_example() -> &'static str {
    match parse_args_example() {
        Ok(val) => val,
        Err(AtError::InvalidArgs) => "Argomento non valido",
        Err(_) => "Errore generico",
    }
}

// In an embedded environment, these functions can be called from main or from a task.

// Mock main for compilation (in real embedded code, this would be in your firmware)
fn main() {
    // Example usage - in embedded this would be called from your main loop
    let _result = handle_error_example();
}
