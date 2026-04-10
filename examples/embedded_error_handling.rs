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
 
//! Advanced example: error handling patterns with the at_response! macro
//!
//! Demonstrates how to propagate and inspect `AtResult` values whose Ok and
//! Err variants both carry the AT response prefix alongside the payload.

#![allow(dead_code)]
#![no_std]
#![no_main]

extern crate at_parser_rs;

use at_parser_rs::context::AtContext;
use at_parser_rs::parser::AtParser;
use at_parser_rs::{Args, AtError, AtResult, at_response};

const SIZE: usize = 64;

/// A command that may fail in different ways
struct DiagModule {
    armed: bool,
}

impl AtContext<SIZE> for DiagModule {
    fn exec(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        if self.armed {
            Ok(at_response!(SIZE, at_response; "TRIGGERED"))
        } else {
            Err((at_response, AtError::Unhandled("not armed")))
        }
    }

    fn query(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; if self.armed { 1u8 } else { 0u8 }))
    }

    fn test(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; "(0,1)"))
    }

    fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<'_, SIZE> {
        let val = args.get(0).ok_or((at_response, AtError::InvalidArgs))?;
        match val.as_ref() {
            "0" => { self.armed = false; Ok(at_response!(SIZE, at_response; "OK")) }
            "1" => { self.armed = true;  Ok(at_response!(SIZE, at_response; "OK")) }
            _ => Err((at_response, AtError::InvalidArgs)),
        }
    }
}

/// Inspect the result of an execute call and return a status byte
fn check_result(result: AtResult<SIZE>) -> u8 {
    match result {
        Ok((_, _))                           => 0,  // success
        Err((_, AtError::InvalidArgs))       => 1,
        Err((_, AtError::NotSupported))      => 2,
        Err((_, AtError::UnknownCommand))    => 3,
        Err((_, AtError::Unhandled(_)))      => 4,
        Err((_, AtError::UnhandledOwned(_))) => 5,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let mut diag = DiagModule { armed: false };

    let mut parser: AtParser<DiagModule, SIZE> = AtParser::new();
    let commands: &mut [(&str, &str, &mut DiagModule)] = &mut [
        ("AT+DIAG", "+DIAG: ", &mut diag),
    ];
    parser.set_commands(commands);

    // Not armed yet: exec returns Unhandled
    let _s = check_result(parser.execute("AT+DIAG"));    // -> 4

    // Arm it
    let _s = check_result(parser.execute("AT+DIAG=1"));  // -> 0

    // Now exec succeeds
    let _s = check_result(parser.execute("AT+DIAG"));    // -> 0

    // Query state
    let _s = check_result(parser.execute("AT+DIAG?"));   // -> 0

    // Invalid argument
    let _s = check_result(parser.execute("AT+DIAG=2"));  // -> 1

    // Unknown command
    let _s = check_result(parser.execute("AT+UNKNOWN")); // -> 3

    loop {}
}

