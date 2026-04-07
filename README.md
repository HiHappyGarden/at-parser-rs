# AT-Parser-RS

A lightweight, `no_std` AT command parser library for embedded Rust applications.

[![Crates.io](https://img.shields.io/crates/v/at-parser-rs.svg)](https://crates.io/crates/at-parser-rs)
[![Documentation](https://docs.rs/at-parser-rs/badge.svg)](https://docs.rs/at-parser-rs)
[![License: LGPL-2.1](https://img.shields.io/badge/License-LGPL%202.1-blue.svg)](LICENSE)

## Overview

AT-Parser-RS provides a flexible framework for implementing AT command interfaces in embedded systems. It supports the standard AT command syntax including execution, query, test, and set operations.

## Features

- `no_std` compatible - suitable for bare-metal and embedded environments
- Fixed-size response buffers via `Bytes<SIZE>` — no heap allocation
- Support for all AT command forms:
  - `AT+CMD` - Execute command
  - `AT+CMD?` - Query current value
  - `AT+CMD=?` - Test supported values
  - `AT+CMD=<args>` - Set new value(s)
- Type-safe command registration via traits
- Static command definitions (suitable for embedded/RTOS)

### Feature Flags

The library supports the following optional features:

- **`osal_rs`** - Enables integration with FreeRTOS through the [osal-rs](https://crates.io/crates/osal-rs) library for RTOS-based applications. Provides synchronization primitives like `Mutex` for thread-safe command handling.
- **`enable_panic`** - Enables a custom panic handler for `no_std` environments, providing a minimal panic implementation for embedded targets.

By default, no features are enabled, providing pure `no_std` compatibility without external dependencies.

```bash
# Build with FreeRTOS support
cargo build --features="osal_rs"

# Build with custom panic handler
cargo build --features="enable_panic"

# Build with both features
cargo build --features="osal_rs,enable_panic"
```

## Command Forms

The parser supports four standard AT command forms:

| Form | Syntax | Purpose | Example |
|------|--------|---------|---------|
| **Execute** | `AT+CMD` | Execute an action | `AT+RST` |
| **Query** | `AT+CMD?` | Get current setting | `AT+ECHO?` |
| **Test** | `AT+CMD=?` | Get supported values | `AT+ECHO=?` |
| **Set** | `AT+CMD=<args>` | Set new value(s) | `AT+ECHO=1` |

> **Note**: All commands must start with the `AT` prefix (e.g., `AT+CMD`, not just `+CMD`). The parser expects the full AT command syntax.

## Core Types

### `AtContext<SIZE>` Trait

The main trait for implementing command handlers. The const generic `SIZE` defines the response buffer size in bytes. Override only the methods your command needs:

```rust
pub trait AtContext<const SIZE: usize> {
    fn exec(&self) -> AtResult<'_, SIZE>;
    fn query(&mut self) -> AtResult<'_, SIZE>;
    fn test(&mut self) -> AtResult<'_, SIZE>;
    fn set(&mut self, args: Args) -> AtResult<'_, SIZE>;
}
```

All methods return `Err(AtError::NotSupported)` by default.

### `AtResult<'a, SIZE>` and `AtError<'a>`

```rust
pub type AtResult<'a, const SIZE: usize> = Result<Bytes<SIZE>, AtError<'a>>;

pub enum AtError<'a> {
    UnknownCommand,        // Command not found
    NotSupported,          // Operation not implemented
    InvalidArgs,           // Invalid argument(s)
    Unhandled(&'a str),    // Error with a borrowed description
    UnhandledOwned(String) // Error with an owned description
}
```

Use `Unhandled` when you have a static or borrowed string literal, and `UnhandledOwned` when you need to construct an error message dynamically at runtime.

### `Bytes<SIZE>`

`Bytes<SIZE>` is a fixed-size byte buffer from `osal-rs` (re-exported by this crate) used to return responses without heap allocation:

```rust
use at_parser_rs::Bytes;

// Create from a string slice (truncated to SIZE if longer)
let response = Bytes::<64>::from_str("OK");
```

### `AtParser<T, SIZE>`

The parser is generic over both the handler type `T` and the response buffer size `SIZE`:

```rust
pub struct AtParser<'a, T, const SIZE: usize>
where
    T: AtContext<SIZE> + ?Sized;
```

### `Args` Structure

Provides access to comma-separated arguments:

```rust
pub struct Args<'a> {
    pub raw: &'a str,
}

impl<'a> Args<'a> {
    pub fn get(&self, index: usize) -> Option<&'a str>;
}
```

## Usage Examples

### 1. Define Command Modules

Implement the `AtContext<SIZE>` trait for your command handlers. Choose a buffer size that fits your largest response string:

```rust
use at_parser_rs::context::AtContext;
use at_parser_rs::{AtResult, AtError, Args};
use osal_rs::utils::Bytes;

const SIZE: usize = 64;

/// Echo command - returns/sets echo state
pub struct EchoModule {
    pub echo: bool,
}

impl AtContext<SIZE> for EchoModule {
    // Execute: return current echo state
    fn exec(&self) -> AtResult<'_, SIZE> {
        if self.echo {
            Ok(Bytes::from_str("ECHO: ON"))
        } else {
            Ok(Bytes::from_str("ECHO: OFF"))
        }
    }

    // Query: return current echo value
    fn query(&mut self) -> AtResult<'_, SIZE> {
        if self.echo { Ok(Bytes::from_str("1")) } else { Ok(Bytes::from_str("0")) }
    }

    // Set: enable/disable echo
    fn set(&mut self, args: Args) -> AtResult<'_, SIZE> {
        let v = args.get(0).ok_or(AtError::InvalidArgs)?;
        match v {
            "0" => {
                self.echo = false;
                Ok(Bytes::from_str("ECHO OFF"))
            }
            "1" => {
                self.echo = true;
                Ok(Bytes::from_str("ECHO ON"))
            }
            _ => Err(AtError::InvalidArgs),
        }
    }

    // Test: show valid values and usage
    fn test(&mut self) -> AtResult<'_, SIZE> {
        Ok(Bytes::from_str("Valid values: 0 (OFF), 1 (ON)"))
    }
}

/// Reset command - executes system reset
pub struct ResetModule;

impl AtContext<SIZE> for ResetModule {
    fn exec(&self) -> AtResult<'_, SIZE> {
        // Trigger hardware reset
        // reset_system();
        Ok(Bytes::from_str("OK - System reset"))
    }

    fn test(&mut self) -> AtResult<'_, SIZE> {
        Ok(Bytes::from_str("Reset the system"))
    }
}
```

### 2. Create Module Instances

For standard applications, create instances on the stack:

```rust
let mut echo = EchoModule { echo: false };
let mut reset = ResetModule;
```

For embedded/`no_std` environments with `static mut` (single-threaded only):

```rust
static mut ECHO: EchoModule = EchoModule { echo: false };
static mut RESET: ResetModule = ResetModule;
```

> **Note**: `static mut` requires `unsafe` blocks and is only safe in single-threaded contexts. For RTOS or multi-threaded applications, use proper synchronization primitives.

### 3. Initialize Parser and Register Commands

```rust
use at_parser_rs::parser::AtParser;
use at_parser_rs::context::AtContext;

const SIZE: usize = 64;

let mut parser: AtParser<dyn AtContext<SIZE>, SIZE> = AtParser::new();

let commands: &mut [(&str, &mut dyn AtContext<SIZE>)] = &mut [
    ("AT+ECHO", &mut echo),
    ("AT+RST", &mut reset),
];

parser.set_commands(commands);
```

### 4. Execute Commands

```rust
// Execute: show current state
match parser.execute("AT+ECHO") {
    Ok(response) => println!("Response: {}", response),  // "ECHO: OFF"
    Err(e) => println!("Error: {:?}", e),
}

// Test: show valid values
match parser.execute("AT+ECHO=?") {
    Ok(response) => println!("Valid: {}", response),     // "Valid values: 0 (OFF), 1 (ON)"
    Err(e) => println!("Error: {:?}", e),
}

// Set: enable echo
match parser.execute("AT+ECHO=1") {
    Ok(response) => println!("Response: {}", response),  // "ECHO ON"
    Err(e) => println!("Error: {:?}", e),
}

// Query: get current value
match parser.execute("AT+ECHO?") {
    Ok(response) => println!("Echo: {}", response),      // "1"
    Err(e) => println!("Error: {:?}", e),
}

// Execute reset
match parser.execute("AT+RST") {
    Ok(response) => println!("Response: {}", response),  // "OK - System reset"
    Err(e) => println!("Error: {:?}", e),
}

// Unknown command
match parser.execute("AT+UNKNOWN") {
    Ok(_) => {},
    Err(AtError::UnknownCommand) => println!("Command not found"),
    Err(_) => {}
}
```

`Bytes<SIZE>` implements `Display`, so it can be printed directly with `{}` or converted to a string via `.to_string()`.

## Advanced Example: UART Module

```rust
use at_parser_rs::{AtResult, AtError, Args};
use osal_rs::utils::Bytes;
use at_parser_rs::context::AtContext;

const SIZE: usize = 64;

pub struct UartModule {
    pub baudrate: u32,
    pub data_bits: u8,
}

impl AtContext<SIZE> for UartModule {
    // Query: return current configuration
    fn query(&mut self) -> AtResult<'_, SIZE> {
        Ok(Bytes::from_str("115200,8"))
    }

    // Set: configure UART
    fn set(&mut self, args: Args) -> AtResult<'_, SIZE> {
        let baudrate = args.get(0)
            .ok_or(AtError::InvalidArgs)?
            .parse::<u32>()
            .map_err(|_| AtError::InvalidArgs)?;
        
        let data_bits = args.get(1)
            .ok_or(AtError::InvalidArgs)?
            .parse::<u8>()
            .map_err(|_| AtError::InvalidArgs)?;

        if ![7, 8].contains(&data_bits) {
            return Err(AtError::InvalidArgs);
        }

        self.baudrate = baudrate;
        self.data_bits = data_bits;
        
        // Apply configuration to hardware
        // configure_uart(baudrate, data_bits);
        
        Ok(Bytes::from_str("OK"))
    }

    // Test: show valid configurations and usage
    fn test(&mut self) -> AtResult<'_, SIZE> {
        Ok(Bytes::from_str("AT+UART=<baudrate>,<data_bits> where baudrate: 9600-115200, data_bits: 7|8"))
    }
}
```

Usage:
```rust
parser.execute("AT+UART=?");        // "AT+UART=<baudrate>,<data_bits> where..."
parser.execute("AT+UART=115200,8"); // "OK"
parser.execute("AT+UART?");         // "115200,8"
```

## Parsing Arguments

The `Args` structure provides a simple interface for accessing comma-separated arguments.
Quoted values are treated as a single argument, so commas inside `"..."` do not split the field.
When a quoted argument contains `\"`, `Args::get()` returns the decoded `"` character:

```rust
fn set(&mut self, args: Args) -> AtResult<'_, SIZE> {
    let arg0 = args.get(0).ok_or(AtError::InvalidArgs)?;
    let arg1 = args.get(1).ok_or(AtError::InvalidArgs)?;
    let arg2 = args.get(2); // Optional argument
    
    // Process arguments...
    Ok(Bytes::from_str("OK"))
}
```

**Important**: `Args::get()` uses 0-based indexing. For a command like `AT+CMD=foo,bar,baz`:
- `args.get(0).as_deref()` returns `Some("foo")`
- `args.get(1).as_deref()` returns `Some("bar")`
- `args.get(2).as_deref()` returns `Some("baz")`
- `args.get(3)` returns `None`

For a command like `AT+SESS=i,"ciao, sono \"antonio\"",mysecretpassword`:
- `args.get(0).as_deref()` returns `Some("i")`
- `args.get(1).as_deref()` returns `Some("ciao, sono \"antonio\"")`
- `args.get_raw(1)` returns `Some("ciao, sono \\\"antonio\\\"")`
- `args.get(2).as_deref()` returns `Some("mysecretpassword")`

For numeric arguments:
```rust
let value = args.get(0)
    .ok_or(AtError::InvalidArgs)?
    .parse::<i32>()
    .map_err(|_| AtError::InvalidArgs)?;
```

Use `Args::get_raw()` only when you explicitly need the original escaped content from a quoted argument:

```rust
let name = args.get(1)
    .ok_or(AtError::InvalidArgs)?;

assert_eq!(name.as_ref(), "ciao, sono \"antonio\"");
```

## Thread Safety

### Single-threaded (bare-metal)
```rust
static mut MODULE: MyModule = MyModule::new();
// Safe in single-threaded context
```

### Multi-threaded (RTOS)
```rust
use core::cell::RefCell;
use osal_rs::sync::Mutex;

static MODULE: Mutex<RefCell<MyModule>> = Mutex::new(RefCell::new(MyModule::new()));
```

## Using the `at_modules!` Macro

The library provides an `at_modules!` macro for defining static command arrays. The first argument is the `SIZE` const:

```rust
use at_parser_rs::at_modules;
use at_parser_rs::context::AtContext;

const SIZE: usize = 64;

static mut ECHO: EchoModule = EchoModule { echo: false };
static mut RESET: ResetModule = ResetModule;

at_modules! {
    SIZE;
    "AT+ECHO" => ECHO,
    "AT+RST" => RESET,
}
```

### Limitations and Considerations

⚠️ **Important**: This macro has significant limitations:

1. **Unsafe**: The macro creates mutable references to static data, requiring `unsafe` blocks
2. **Single-threaded only**: Not suitable for multi-threaded or RTOS environments
3. **Limited flexibility**: Cannot mix different command handler types

### Recommended Alternative

For most applications, the manual approach shown in the examples is preferred:

```rust
use at_parser_rs::context::AtContext;
use at_parser_rs::parser::AtParser;

const SIZE: usize = 64;

let mut echo = EchoModule { echo: false };
let mut reset = ResetModule;

let commands: &mut [(&str, &mut dyn AtContext<SIZE>)] = &mut [
    ("AT+ECHO", &mut echo),
    ("AT+RST", &mut reset),
];

parser.set_commands(commands);
```

This approach is safer, more flexible, and works in all contexts (stack, heap, RTOS).

## Best Practices

1. **Choose an appropriate `SIZE`**: Pick a buffer size that fits your largest response string; responses longer than `SIZE` are silently truncated
2. **Validate arguments**: Always check argument count and validity before processing
3. **Handle errors gracefully**: Use appropriate `AtError` variants for different failure modes. Use `AtError::Unhandled("msg")` for static string descriptions and `AtError::UnhandledOwned(string)` for dynamically constructed messages
4. **Document test responses**: Use `test()` to provide clear usage information
5. **Minimize state**: Keep module state simple and thread-safe

## Examples

The library includes several example files demonstrating different usage patterns:

### Standard Examples
- **`complete_usage.rs`** - Complete demonstration with multiple command types (Echo, Reset, Info, LED)
- **`basic_parser.rs`** - Shows direct usage of the `AtParser` with comprehensive test cases

### Embedded/no_std Examples

These examples demonstrate code patterns suitable for `no_std` environments:

- **`embedded_basic.rs`** - Basic patterns and error handling for no_std/embedded environments
- **`embedded_error_handling.rs`** - Patterns for custom error handling and type conversions
- **`embedded_uart_config.rs`** - UART and device configuration patterns with `AtContext` implementation

> **Note**: The embedded examples are designed to show code patterns and best practices rather than being fully functional standalone programs. They demonstrate how to structure code for embedded/no_std contexts.

Run examples with:
```bash
# Standard examples (fully functional)
cargo run --example complete_usage
cargo run --example basic_parser

# Embedded examples (demonstrate patterns)
cargo run --example embedded_basic --no-default-features
cargo run --example embedded_error_handling --no-default-features
cargo run --example embedded_uart_config --no-default-features
```

## License

This project is licensed under the GNU Lesser General Public License v2.1 or later (LGPL-2.1-or-later) - see the [LICENSE](LICENSE) file for details.
