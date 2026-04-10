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

- **`freertos`** (default) — Enable FreeRTOS support via [osal-rs](https://crates.io/crates/osal-rs).
- **`posix`** — Enable POSIX (Linux/macOS) threading support via osal-rs.
- **`std`** — Enable standard library support via osal-rs.
- **`disable_panic`** — Pass-through feature to osal-rs; disables the built-in panic handler.

By default the `freertos` feature is enabled.

```bash
# Build with FreeRTOS support (default)
cargo build

# Build with POSIX support
cargo build --no-default-features --features="posix"

# Build with std support
cargo build --no-default-features --features="std"

# Disable the default panic handler
cargo build --features="disable_panic"
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
    fn exec(&mut self, at_response: &'static str) -> AtResult<'_, SIZE>;
    fn query(&mut self, at_response: &'static str) -> AtResult<'_, SIZE>;
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, SIZE>;
    fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<'_, SIZE>;
}
```

The `at_response` parameter is the AT response prefix string (e.g. `"+ECHO: "`) that was
registered alongside the command. Pass it through to `Ok(...)` / `Err(...)` so the caller
can format the full response line. Use the [`at_response!`](#at_response-macro) macro for
convenient formatting.

All methods return `Err((at_response, AtError::NotSupported))` by default.

### `AtResult<'a, SIZE>` and `AtError<'a>`

```rust
// Both Ok and Err carry the AT response prefix together with the payload
pub type AtResult<'a, const SIZE: usize> =
    Result<(&'static str, Bytes<SIZE>), (&'static str, AtError<'a>)>;

pub enum AtError<'a> {
    UnknownCommand,        // Command not found
    NotSupported,          // Operation not implemented
    InvalidArgs,           // Invalid argument(s)
    Unhandled(&'a str),    // Error with a borrowed description
    UnhandledOwned(String) // Error with an owned description
}
```

The first element of the tuple is always the AT response prefix (`at_response`) received from
the parser, so callers can reconstruct the full response line regardless of whether the
call succeeded or failed.

Use `Unhandled` when you have a static string literal, and `UnhandledOwned` when you need
to construct an error message dynamically at runtime.

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

Commands are registered as **3-tuples**: `(at_command, at_response, handler)` where
`at_command` is the string the parser matches against (e.g. `"AT+ECHO"`) and
`at_response` is the prefix forwarded to the handler (e.g. `"+ECHO: "`). These can be the
same string or different—choose whatever your protocol requires.

### `Args` Structure

Provides access to comma-separated arguments:

```rust
pub struct Args<'a> {
    pub raw: &'a str,
}

impl<'a> Args<'a> {
    /// Returns the n-th argument, unquoting and decoding escape sequences.
    pub fn get(&self, index: usize) -> Option<Cow<'a, str>>;
    /// Returns the n-th argument as-is (no escape decoding).
    pub fn get_raw(&self, index: usize) -> Option<&'a str>;
}
```

## Usage Examples

### 1. Define Command Modules

Implement the `AtContext<SIZE>` trait for your command handlers. Choose a buffer size that fits your largest response string.

Every method receives the `at_response` prefix that was registered for this command so you
can include it in the response (use the `at_response!` macro for convenience):

```rust
use at_parser_rs::context::AtContext;
use at_parser_rs::{AtResult, AtError, Args, at_response};
use osal_rs::utils::Bytes;

const SIZE: usize = 64;

/// Echo command - returns/sets echo state
pub struct EchoModule {
    pub echo: bool,
}

impl AtContext<SIZE> for EchoModule {
    // Execute: return current echo state
    fn exec(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        let state: u8 = if self.echo { 1 } else { 0 };
        Ok(at_response!(SIZE, at_response; state))
    }

    // Query: return current echo value
    fn query(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; if self.echo { 1u8 } else { 0u8 }))
    }

    // Set: enable/disable echo
    fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<'_, SIZE> {
        let v = args.get(0).ok_or((at_response, AtError::InvalidArgs))?;
        match v.as_ref() {
            "0" => { self.echo = false; Ok(at_response!(SIZE, at_response; "OK")) }
            "1" => { self.echo = true;  Ok(at_response!(SIZE, at_response; "OK")) }
            _ => Err((at_response, AtError::InvalidArgs)),
        }
    }

    // Test: show valid values and usage
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; "(0,1)"))
    }
}

/// Reset command - executes system reset
pub struct ResetModule;

impl AtContext<SIZE> for ResetModule {
    fn exec(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        // Trigger hardware reset here if needed
        Ok(at_response!(SIZE, at_response; "OK"))
    }

    fn test(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; "Reset the system"))
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

Commands are registered as 3-tuples: `(at_command, at_response_prefix, handler)`.

```rust
use at_parser_rs::parser::AtParser;
use at_parser_rs::context::AtContext;

const SIZE: usize = 64;

let mut parser: AtParser<dyn AtContext<SIZE>, SIZE> = AtParser::new();

let commands: &mut [(&str, &str, &mut dyn AtContext<SIZE>)] = &mut [
    ("AT+ECHO", "+ECHO: ", &mut echo),
    ("AT+RST",  "+RST: ",  &mut reset),
];

parser.set_commands(commands);
```

### 4. Execute Commands

`execute` returns `Ok((prefix, bytes))` on success or `Err((prefix, error))` on failure,
where `prefix` is the AT response prefix registered for that command.

```rust
// Execute: return current state
match parser.execute("AT+ECHO") {
    Ok((prefix, response)) => println!("{}{}", prefix, response),  // "+ECHO: 0"
    Err((prefix, e)) => println!("{} ERROR: {:?}", prefix, e),
}

// Test: show valid values
match parser.execute("AT+ECHO=?") {
    Ok((prefix, response)) => println!("{}{}", prefix, response),  // "+ECHO: (0,1)"
    Err((prefix, e)) => println!("{} ERROR: {:?}", prefix, e),
}

// Set: enable echo
match parser.execute("AT+ECHO=1") {
    Ok((prefix, response)) => println!("{}{}", prefix, response),  // "+ECHO: OK"
    Err((prefix, e)) => println!("{} ERROR: {:?}", prefix, e),
}

// Query: get current value
match parser.execute("AT+ECHO?") {
    Ok((prefix, response)) => println!("{}{}", prefix, response),  // "+ECHO: 1"
    Err((prefix, e)) => println!("{} ERROR: {:?}", prefix, e),
}

// Unknown command → Err(("" , AtError::UnknownCommand))
match parser.execute("AT+UNKNOWN") {
    Ok(_) => {},
    Err((_, AtError::UnknownCommand)) => println!("Command not found"),
    Err(_) => {}
}
```

`Bytes<SIZE>` implements `Display`, so it can be printed directly with `{}` or converted
to a string via `.to_string()`.

## Advanced Example: UART Module

```rust
use at_parser_rs::{AtResult, AtError, Args, at_response};
use at_parser_rs::context::AtContext;

const SIZE: usize = 64;

pub struct UartModule {
    pub baudrate: u32,
    pub data_bits: u8,
}

impl AtContext<SIZE> for UartModule {
    // Query: return current configuration
    fn query(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; self.baudrate, self.data_bits))
    }

    // Set: configure UART
    fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<'_, SIZE> {
        let baudrate = args.get(0)
            .ok_or((at_response, AtError::InvalidArgs))?
            .parse::<u32>()
            .map_err(|_| (at_response, AtError::InvalidArgs))?;

        let data_bits = args.get(1)
            .ok_or((at_response, AtError::InvalidArgs))?
            .parse::<u8>()
            .map_err(|_| (at_response, AtError::InvalidArgs))?;

        if ![7u8, 8].contains(&data_bits) {
            return Err((at_response, AtError::InvalidArgs));
        }

        self.baudrate = baudrate;
        self.data_bits = data_bits;
        // configure_uart(baudrate, data_bits);

        Ok(at_response!(SIZE, at_response; "OK"))
    }

    // Test: show valid configurations
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, SIZE> {
        Ok(at_response!(SIZE, at_response; "<baudrate: 9600-115200>,<data_bits: 7|8>"))
    }
}
```

Usage:
```rust
// Register: ("AT+UART", "+UART: ", &mut uart)
parser.execute("AT+UART=?");        // Ok(("+UART: ", "<baudrate: 9600-115200>,<data_bits: 7|8>"))
parser.execute("AT+UART=115200,8"); // Ok(("+UART: ", "OK"))
parser.execute("AT+UART?");         // Ok(("+UART: ", "115200,8"))
```

## Parsing Arguments

The `Args` structure provides a simple interface for accessing comma-separated arguments.
Quoted values are treated as a single argument, so commas inside `"..."` do not split the field.
When a quoted argument contains `\"`, `Args::get()` returns the decoded `"` character:

```rust
fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<'_, SIZE> {
    let arg0 = args.get(0).ok_or((at_response, AtError::InvalidArgs))?;
    let arg1 = args.get(1).ok_or((at_response, AtError::InvalidArgs))?;
    let arg2 = args.get(2); // Optional argument

    // Process arguments...
    Ok(at_response!(SIZE, at_response; "OK"))
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
    .ok_or((at_response, AtError::InvalidArgs))?
    .parse::<i32>()
    .map_err(|_| (at_response, AtError::InvalidArgs))?;
```

Use `Args::get_raw()` only when you explicitly need the original escaped content from a
quoted argument:

```rust
let name = args.get(1)
    .ok_or((at_response, AtError::InvalidArgs))?;

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

## `at_response!` Macro

Constructs an `Ok((&'static str, Bytes<SIZE>))` value from a response prefix and 1–6
comma-separated arguments:

```rust
use at_parser_rs::at_response;

const SIZE: usize = 64;

// Single value
let r = at_response!(SIZE, "+ECHO: "; 1u8);              // ("+ECHO: ", "1")

// Two values
let r = at_response!(SIZE, "+LED: "; 1u8, 75u8);         // ("+LED: ", "1,75")

// Three values
let r = at_response!(SIZE, "+NET: "; "192.168.1.1", 8080u16, 1u8);
```

## `at_quoted!` Macro

Wraps a value in double-quote characters, useful inside `at_response!` when the
protocol requires quoted strings:

```rust
use at_parser_rs::{at_response, at_quoted};

const SIZE: usize = 64;
let ssid = "MyNetwork";
let r = at_response!(SIZE, "+WIFI: "; at_quoted!(ssid), -70i8);
// ("+WIFI: ", "\"MyNetwork\",-70")
```

## Using the `at_modules!` Macro

The library provides an `at_modules!` macro for defining static command arrays.
Each entry is a 3-tuple: `(at_command, at_response) => HANDLER`.

```rust
use at_parser_rs::at_modules;
use at_parser_rs::context::AtContext;

const SIZE: usize = 64;

static mut ECHO:  EchoModule  = EchoModule { echo: false };
static mut RESET: ResetModule = ResetModule;

at_modules! {
    SIZE;
    ("AT+ECHO", "+ECHO: ") => ECHO,
    ("AT+RST",  "+RST: ")  => RESET,
}
// COMMANDS is now available: parser.set_commands(COMMANDS);
```

### Limitations and Considerations

⚠️ **Important**: This macro has significant limitations:

1. **Unsafe**: The macro creates mutable references to static data, requiring `unsafe` blocks
2. **Single-threaded only**: Not suitable for multi-threaded or RTOS environments
3. **Limited flexibility**: Cannot mix different command handler types

### Recommended Alternative

For most applications, the manual slice approach is preferred:

```rust
use at_parser_rs::context::AtContext;
use at_parser_rs::parser::AtParser;

const SIZE: usize = 64;

let mut echo  = EchoModule { echo: false };
let mut reset = ResetModule;

let commands: &mut [(&str, &str, &mut dyn AtContext<SIZE>)] = &mut [
    ("AT+ECHO", "+ECHO: ", &mut echo),
    ("AT+RST",  "+RST: ",  &mut reset),
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
