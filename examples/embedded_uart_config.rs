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

//! Example: AT command table with UART and device configuration handling
//!
//! Demonstrates `AtContext` implementations for device-specific commands
//! in a no_std/embedded environment, using the `at_response!` macro and
//! the 3-tuple command registration `(at_command, at_response, handler)`.

#![allow(dead_code)]
#![no_std]
#![no_main]

extern crate at_parser_rs;

use at_parser_rs::context::AtContext;
use at_parser_rs::parser::AtParser;
use at_parser_rs::{Args, AtError, AtResult, at_response};

const SIZE: usize = 64;

/// UART send command — forwards data to the hardware peripheral
struct UartSendModule {
    baudrate: u32,
}

impl UartSendModule {
    const fn new() -> Self {
        Self { baudrate: 9600 }
    }

    fn write(&self, _data: &str) {
        // In real embedded code: write to the UART peripheral here
    }
}

impl AtContext<SIZE> for UartSendModule {
    /// Query: return current baud-rate
    fn query(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; self.baudrate))
    }

    /// Test: show accepted syntax
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; "\"<data>\""))
    }

    /// Set: send the given data string over UART
    fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<'_, SIZE> {
        let data = args.get(0).ok_or((at_response, AtError::InvalidArgs))?;
        self.write(data.as_ref());
        Ok(at_response!(SIZE, at_response; "SENT"))
    }
}

/// Device configuration command — get/set baudrate and mode
struct ConfigModule {
    baudrate: u32,
    mode: u8,
}

impl ConfigModule {
    const fn new() -> Self {
        Self { baudrate: 115200, mode: 0 }
    }
}

impl AtContext<SIZE> for ConfigModule {
    /// Query: return current configuration as `<baudrate>,<mode>`
    fn query(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; self.baudrate, self.mode))
    }

    /// Test: return supported configuration ranges
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; "<baudrate: 9600-115200>,<mode: 0|1>"))
    }

    /// Set: apply new baudrate and mode
    fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<'_, SIZE> {
        let baudrate = args.get(0)
            .ok_or((at_response, AtError::InvalidArgs))?
            .parse::<u32>()
            .map_err(|_| (at_response, AtError::InvalidArgs))?;

        let mode = args.get(1)
            .ok_or((at_response, AtError::InvalidArgs))?
            .parse::<u8>()
            .map_err(|_| (at_response, AtError::InvalidArgs))?;

        if mode > 1 {
            return Err((at_response, AtError::InvalidArgs));
        }

        self.baudrate = baudrate;
        self.mode = mode;
        // configure_uart(baudrate, mode);  // apply to hardware here

        Ok(at_response!(SIZE, at_response; "OK"))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let mut uart   = UartSendModule::new();
    let mut config = ConfigModule::new();

    let mut parser: AtParser<dyn AtContext<SIZE>, SIZE> = AtParser::new();
    let commands: &mut [(&str, &str, &mut dyn AtContext<SIZE>)] = &mut [
        ("AT+UARTSEND", "+UARTSEND: ", &mut uart),
        ("AT+CFG",      "+CFG: ",      &mut config),
    ];
    parser.set_commands(commands);

    // Test: show syntax
    let _ = parser.execute("AT+UARTSEND=?"); // Ok(("+UARTSEND: ", "\"<data>\""))
    let _ = parser.execute("AT+CFG=?");      // Ok(("+CFG: ", "<baudrate: 9600-115200>,<mode: 0|1>"))

    // Query current config
    let _ = parser.execute("AT+CFG?");       // Ok(("+CFG: ", "115200,0"))

    // Send data over UART
    let _ = parser.execute("AT+UARTSEND=\"hello\""); // Ok(("+UARTSEND: ", "SENT"))

    // Set new configuration
    let _ = parser.execute("AT+CFG=9600,1"); // Ok(("+CFG: ", "OK"))
    let _ = parser.execute("AT+CFG?");       // Ok(("+CFG: ", "9600,1"))

    // Error cases
    let _ = parser.execute("AT+CFG=9600,5"); // Err(("+CFG: ", InvalidArgs))  — mode > 1
    let _ = parser.execute("AT+CFG=abc,0");  // Err(("+CFG: ", InvalidArgs))  — bad baudrate

    loop {}
}

