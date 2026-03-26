/***************************************************************************
 *
 * AT Command Parser
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/
 
//! Basic usage example demonstrating no_std compatible code
//! This example shows how the parser can be used in no_std contexts

#![allow(dead_code)]

extern crate at_parser_rs;

use at_parser_rs::{Args, AtError, AtResult};

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
