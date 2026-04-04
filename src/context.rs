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
 
use crate::{Args, AtError, AtResult};

/// Trait that defines the context for AT command execution.
/// Implementations of this trait handle the actual logic for each AT command form.
///
/// The const generic `SIZE` defines the size (in bytes) of the response buffer
/// returned by command handlers.
pub trait AtContext<const SIZE: usize> {

    /// Execute command (AT+CMD)
    /// This is called when a command is invoked without any suffix.
    fn exec(&mut self) -> AtResult<'_, SIZE> {
        Err(AtError::NotSupported)
    }

    /// Query command (AT+CMD?)
    /// This is called to retrieve the current value/state of a command.
    fn query(&mut self) -> AtResult<'_, SIZE> {
        Err(AtError::NotSupported)
    }
    
    /// Test command (AT+CMD=?)
    /// This is called to check if a command is supported or to get valid parameter ranges.
    fn test(&mut self) -> AtResult<'_, SIZE> {
        Err(AtError::NotSupported)
    }

    /// Set command (AT+CMD=args)
    /// This is called to set parameters for a command.
    fn set(&mut self, _args: Args) -> AtResult<'_, SIZE> {
        Err(AtError::NotSupported)
    }

}