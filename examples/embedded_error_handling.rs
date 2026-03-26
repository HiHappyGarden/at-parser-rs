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
 
//! Advanced example: macro usage and custom error handling
//! This example demonstrates code patterns suitable for no_std environments

#![allow(dead_code)]

extern crate at_parser_rs;

use at_parser_rs::{Args, AtError};

// Example macro to register commands (mock, since AtContext is missing)
macro_rules! dummy_at_modules {
    ($($name:expr => $module:expr),* $(,)?) => {
        // In a real case, modules would be registered here
        // In no_std, be careful not to use heap or mutable statics without sync
    };
}

dummy_at_modules! {
    "CMD1" => 1,
    "CMD2" => 2,
}

// Function that simulates AT command handling
fn handle_at_command<'a>(cmd: &str, args: &'a str) -> Result<&'a str, AtError> {
    match cmd {
        "CMD1" => {
            let a = Args { raw: args };
            a.get(0).ok_or(AtError::InvalidArgs)
        }
        "CMD2" => Ok("OK"),
        _ => Err(AtError::UnknownCommand),
    }
}

// Example call
fn example_usage() -> &'static str {
    match handle_at_command("CMD1", "foo,bar") {
        Ok(val) => val,
        Err(_) => "Errore",
    }
}

// Mock main for compilation (in real embedded code, this would be in your firmware)
fn main() {
    // Example usage - in embedded this would be called from your main loop
    let _result = example_usage();
}
