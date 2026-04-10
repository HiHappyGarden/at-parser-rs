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
 
//! Complete example demonstrating the AT command parser functionality
//!
//! **Note**: no_std compatible example - designed to compile and run on embedded
//! targets. Demonstrates all AT command forms without std dependency.

#![no_std]
#![no_main]
#![allow(dead_code, unused_variables)]

extern crate at_parser_rs;

use at_parser_rs::context::AtContext;
use at_parser_rs::parser::AtParser;
use at_parser_rs::{Args, AtError, AtResult, at_response};

const SIZE: usize = 64;

/// Echo command module - manages echo state
pub struct EchoModule {
    pub echo: bool,
}

impl AtContext<SIZE> for EchoModule {
    /// Execute: return current echo state
    fn exec(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; if self.echo { "ON" } else { "OFF" }))
    }

    /// Query: return current echo value (0 or 1)
    fn query(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; if self.echo { 1u8 } else { 0u8 }))
    }

    /// Test: show valid values
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; "(0,1)"))
    }

    /// Set: enable/disable echo
    fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<'_, SIZE> {
        let value = args.get(0).ok_or((at_response, AtError::InvalidArgs))?;
        match value.as_ref() {
            "0" => {
                self.echo = false;
                Ok(at_response!(SIZE, at_response; "OK"))
            }
            "1" => {
                self.echo = true;
                Ok(at_response!(SIZE, at_response; "OK"))
            }
            _ => Err((at_response, AtError::InvalidArgs)),
        }
    }
}

/// Reset command module - simulates system reset
pub struct ResetModule;

impl AtContext<SIZE> for ResetModule {
    /// Execute: perform reset
    fn exec(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; "OK"))
    }

    /// Test: show command description
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; "Reset the system"))
    }
}

/// Info command module - provides system information
pub struct InfoModule {
    pub version: &'static str,
}

impl AtContext<SIZE> for InfoModule {
    /// Execute: return version string
    fn exec(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; self.version))
    }

    /// Query: return detailed info
    fn query(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; "AT-Parser-RS - AT Command Parser Library"))
    }
}

/// LED command module - controls an LED with multiple parameters
pub struct LedModule {
    pub state: bool,
    pub brightness: u8,
}

impl AtContext<SIZE> for LedModule {
    /// Execute: return current LED state
    fn exec(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; if self.state { "ON" } else { "OFF" }))
    }

    /// Query: return state and brightness
    fn query(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; self.state as u8, self.brightness))
    }

    /// Test: show usage
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; "<state: 0|1>,<brightness: 0-100>"))
    }

    /// Set: change LED state and brightness
    fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<'_, SIZE> {
        let state_str = args.get(0).ok_or((at_response, AtError::InvalidArgs))?;

        self.state = match state_str.as_ref() {
            "0" => false,
            "1" => true,
            _ => return Err((at_response, AtError::InvalidArgs)),
        };

        // Optional brightness parameter
        if let Some(brightness_str) = args.get(1) {
            let bri = brightness_str
                .parse::<u8>()
                .map_err(|_| (at_response, AtError::InvalidArgs))?;

            if bri > 100 {
                return Err((at_response, AtError::InvalidArgs));
            }
            self.brightness = bri;
        }

        Ok(at_response!(SIZE, at_response; "OK"))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let mut echo = EchoModule { echo: false };
    let mut reset = ResetModule;
    let mut info = InfoModule { version: "v1.0.0" };
    let mut led = LedModule { state: false, brightness: 0 };

    let mut parser: AtParser<dyn AtContext<SIZE>, SIZE> = AtParser::new();

    let commands: &mut [(&str, &str, &mut dyn AtContext<SIZE>)] = &mut [
        ("AT+ECHO", "+ECHO: ", &mut echo),
        ("AT+RST",  "+RST: ",  &mut reset),
        ("AT+INFO", "+INFO: ", &mut info),
        ("AT+LED",  "+LED: ",  &mut led),
    ];
    parser.set_commands(commands);

    // INFO
    let _ = parser.execute("AT+INFO");   // Ok(("+INFO: ", "v1.0.0"))
    let _ = parser.execute("AT+INFO?");  // Ok(("+INFO: ", "AT-Parser-RS ..."))

    // ECHO
    let _ = parser.execute("AT+ECHO");   // Ok(("+ECHO: ", "OFF"))
    let _ = parser.execute("AT+ECHO=?"); // Ok(("+ECHO: ", "(0,1)"))
    let _ = parser.execute("AT+ECHO=1"); // Ok(("+ECHO: ", "OK"))
    let _ = parser.execute("AT+ECHO?");  // Ok(("+ECHO: ", "1"))
    let _ = parser.execute("AT+ECHO=0"); // Ok(("+ECHO: ", "OK"))

    // LED
    let _ = parser.execute("AT+LED=?");    // Ok(("+LED: ", "<state: 0|1>,<brightness: 0-100>"))
    let _ = parser.execute("AT+LED=1");    // Ok(("+LED: ", "OK"))
    let _ = parser.execute("AT+LED?");     // Ok(("+LED: ", "1,0"))
    let _ = parser.execute("AT+LED=1,75"); // Ok(("+LED: ", "OK"))
    let _ = parser.execute("AT+LED=0");    // Ok(("+LED: ", "OK"))

    // RESET
    let _ = parser.execute("AT+RST=?"); // Ok(("+RST: ", "Reset the system"))
    let _ = parser.execute("AT+RST");   // Ok(("+RST: ", "OK"))

    // Error cases
    let _ = parser.execute("AT+ECHO=2");   // Err(("+ECHO: ", InvalidArgs))
    let _ = parser.execute("AT+UNKNOWN");  // Err(("", UnknownCommand))

    loop {}
}

