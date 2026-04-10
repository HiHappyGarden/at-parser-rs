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
//!
//! Shows Args parsing and error handling patterns with the updated AtResult
//! tuple type `Result<(&'static str, Bytes<SIZE>), (&'static str, AtError)>`.

#![allow(dead_code)]
#![no_std]
#![no_main]

extern crate at_parser_rs;

use at_parser_rs::{Args, AtError, AtResult, at_response};

const SIZE: usize = 64;
const AT_RESP: &str = "+DEMO: ";

// Parse the second argument and echo it back in the response
fn parse_args_example() -> AtResult<'static, SIZE> {
    let args = Args { raw: "foo,bar,baz" };
    match args.get(1) {
        Some(val) => Ok(at_response!(SIZE, AT_RESP; val.as_ref())),
        None => Err((AT_RESP, AtError::InvalidArgs)),
    }
}

// Demonstrate matching on the new tuple error
fn handle_error_example() -> &'static str {
    match parse_args_example() {
        Ok((_, _)) => "OK",
        Err((_, AtError::InvalidArgs)) => "Argomento non valido",
        Err((_, AtError::UnknownCommand)) => "Comando sconosciuto",
        Err(_) => "Errore generico",
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let _result = handle_error_example();
    loop {}
}

