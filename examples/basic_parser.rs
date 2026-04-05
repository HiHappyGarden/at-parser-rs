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
 
//! Example using the AtParser with proper type handling
//!
//! **Note**: no_std compatible example - designed to compile and run on embedded
//! targets. Demonstrates AtParser usage patterns without std dependency.

#![no_std]
#![no_main]
#![allow(dead_code, unused_variables)]

extern crate at_parser_rs;

use at_parser_rs::context::AtContext;
use at_parser_rs::parser::AtParser;
use at_parser_rs::{Args, AtError, AtResult, Bytes};

const SIZE: usize = 64;

/// Simple command module for testing
pub struct TestCommand {
    pub value: u32,
}

impl AtContext<SIZE> for TestCommand {
    fn exec(&mut self) -> AtResult<'_, SIZE> {
        Ok(Bytes::from_str("Test command executed"))
    }

    fn query(&mut self) -> AtResult<'_, SIZE> {
        if self.value == 0 {
            Ok(Bytes::from_str("0"))
        } else if self.value < 10 {
            Ok(Bytes::from_str("1-9"))
        } else {
            Ok(Bytes::from_str("10+"))
        }
    }

    fn test(&mut self) -> AtResult<'_, SIZE> {
        Ok(Bytes::from_str("Test: 0-100"))
    }

    fn set(&mut self, args: Args) -> AtResult<'_, SIZE> {
        let val_str = args.get(0).ok_or(AtError::InvalidArgs)?;
        self.value = val_str.parse().map_err(|_| AtError::InvalidArgs)?;
        Ok(Bytes::from_str("OK"))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let mut cmd1 = TestCommand { value: 0 };
    let mut cmd2 = TestCommand { value: 5 };
    let mut cmd3 = TestCommand { value: 10 };

    let mut parser: AtParser<TestCommand, SIZE> = AtParser::new();

    let commands: &mut [(&str, &mut TestCommand)] = &mut [
        ("AT+CMD1", &mut cmd1),
        ("AT+CMD2", &mut cmd2),
        ("AT+CMD3", &mut cmd3),
    ];
    parser.set_commands(commands);

    // Execute (no-op result, just exercising the API)
    let _ = parser.execute("AT+CMD1");
    let _ = parser.execute("AT+CMD1?");
    let _ = parser.execute("AT+CMD1=?");
    let _ = parser.execute("AT+CMD1=42");
    let _ = parser.execute("AT+CMD1?");
    let _ = parser.execute("AT+CMD2");
    let _ = parser.execute("AT+CMD2?");
    let _ = parser.execute("AT+CMD3=100");
    let _ = parser.execute("AT+CMD3?");
    let _ = parser.execute("AT+UNKNOWN");   // -> Err(UnknownCommand)
    let _ = parser.execute("AT+CMD1=abc");  // -> Err(InvalidArgs)

    loop {}
}
