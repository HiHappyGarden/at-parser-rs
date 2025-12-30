//! Complete example demonstrating the AT command parser functionality

use parser_rs::context::AtContext;
use parser_rs::{Args, AtError, AtResult};

/// Echo command module - manages echo state
pub struct EchoModule {
    pub echo: bool,
}

impl AtContext for EchoModule {
    /// Execute: return current echo state
    fn exec(&self) -> AtResult<'static> {
        if self.echo {
            Ok("ECHO: ON")
        } else {
            Ok("ECHO: OFF")
        }
    }

    /// Query: return current echo value
    fn query(&mut self) -> AtResult<'static> {
        if self.echo {
            Ok("1")
        } else {
            Ok("0")
        }
    }

    /// Test: show valid values
    fn test(&mut self) -> AtResult<'static> {
        Ok("Valid values: 0 (OFF), 1 (ON)")
    }

    /// Set: enable/disable echo
    fn set(&mut self, args: Args) -> AtResult<'static> {
        let value = args.get(0).ok_or(AtError::InvalidArgs)?;
        match value {
            "0" => {
                self.echo = false;
                Ok("ECHO OFF")
            }
            "1" => {
                self.echo = true;
                Ok("ECHO ON")
            }
            _ => Err(AtError::InvalidArgs),
        }
    }
}

/// Reset command module - simulates system reset
pub struct ResetModule;

impl AtContext for ResetModule {
    /// Execute: perform reset
    fn exec(&self) -> AtResult<'static> {
        println!("  [System reset triggered]");
        Ok("OK - System reset")
    }

    /// Test: show command description
    fn test(&mut self) -> AtResult<'static> {
        Ok("Reset the system")
    }
}

/// Info command module - provides system information
pub struct InfoModule {
    pub version: &'static str,
}

impl AtContext for InfoModule {
    /// Execute: return system info
    fn exec(&self) -> AtResult<'static> {
        Ok(self.version)
    }

    /// Query: return detailed info
    fn query(&mut self) -> AtResult<'static> {
        Ok("Parser-RS v1.0.0 - AT Command Parser Library")
    }
}

/// LED command module - controls an LED with multiple parameters
pub struct LedModule {
    pub state: bool,
    pub brightness: u8,
}

impl AtContext for LedModule {
    /// Execute: return current LED state
    fn exec(&self) -> AtResult<'static> {
        if self.state {
            Ok("LED: ON")
        } else {
            Ok("LED: OFF")
        }
    }

    /// Query: return state and brightness
    fn query(&mut self) -> AtResult<'static> {
        if self.state {
            Ok("1,100")
        } else {
            Ok("0,0")
        }
    }

    /// Test: show usage
    fn test(&mut self) -> AtResult<'static> {
        Ok("AT+LED=<state>,<brightness> where state: 0|1, brightness: 0-100")
    }

    /// Set: change LED state and brightness
    fn set(&mut self, args: Args) -> AtResult<'static> {
        let state_str = args.get(0).ok_or(AtError::InvalidArgs)?;
        
        self.state = match state_str {
            "0" => false,
            "1" => true,
            _ => return Err(AtError::InvalidArgs),
        };

        // Optional brightness parameter
        if let Some(brightness_str) = args.get(1) {
            self.brightness = brightness_str
                .parse::<u8>()
                .map_err(|_| AtError::InvalidArgs)?;
            
            if self.brightness > 100 {
                return Err(AtError::InvalidArgs);
            }
        }

        if self.state {
            Ok("LED ON")
        } else {
            Ok("LED OFF")
        }
    }
}

/// Helper function to execute a command and print the result
fn execute_command(cmd: &str, name: &str, module: &mut dyn AtContext) {
    println!("\n> {}", cmd);
    
    let result = if let Some(rest) = cmd.strip_prefix(name) {
        if rest.is_empty() {
            // Execute form: AT+CMD
            module.exec()
        } else if rest == "?" {
            // Query form: AT+CMD?
            module.query()
        } else if rest == "=?" {
            // Test form: AT+CMD=?
            module.test()
        } else if let Some(args_str) = rest.strip_prefix('=') {
            // Set form: AT+CMD=args
            module.set(Args { raw: args_str })
        } else {
            Err(AtError::InvalidArgs)
        }
    } else {
        Err(AtError::UnknownCommand)
    };
    
    match result {
        Ok(response) => println!("  Response: {}", response),
        Err(AtError::UnknownCommand) => println!("  Error: Unknown command"),
        Err(AtError::NotSupported) => println!("  Error: Operation not supported"),
        Err(AtError::InvalidArgs) => println!("  Error: Invalid arguments"),
    }
}

fn main() {
    println!("=== AT Command Parser Example ===\n");
    println!("Available commands: AT+ECHO, AT+RST, AT+INFO, AT+LED\n");

    // Create module instances
    let mut echo = EchoModule { echo: false };
    let mut reset = ResetModule;
    let mut info = InfoModule { version: "v1.0.0" };
    let mut led = LedModule {
        state: false,
        brightness: 0,
    };

    // Demonstrate all command forms
    
    // 1. INFO command examples
    println!("--- INFO Command ---");
    execute_command("AT+INFO", "AT+INFO", &mut info);      // Execute
    execute_command("AT+INFO?", "AT+INFO", &mut info);     // Query

    // 2. ECHO command examples
    println!("\n--- ECHO Command ---");
    execute_command("AT+ECHO", "AT+ECHO", &mut echo);      // Execute (current state)
    execute_command("AT+ECHO=?", "AT+ECHO", &mut echo);    // Test (show valid values)
    execute_command("AT+ECHO=1", "AT+ECHO", &mut echo);    // Set (enable)
    execute_command("AT+ECHO?", "AT+ECHO", &mut echo);     // Query (check state)
    execute_command("AT+ECHO", "AT+ECHO", &mut echo);      // Execute (should show ON)
    execute_command("AT+ECHO=0", "AT+ECHO", &mut echo);    // Set (disable)
    execute_command("AT+ECHO", "AT+ECHO", &mut echo);      // Execute (should show OFF)

    // 3. LED command examples
    println!("\n--- LED Command ---");
    execute_command("AT+LED=?", "AT+LED", &mut led);       // Test (show usage)
    execute_command("AT+LED=1", "AT+LED", &mut led);       // Set (turn on)
    execute_command("AT+LED?", "AT+LED", &mut led);        // Query
    execute_command("AT+LED=1,75", "AT+LED", &mut led);    // Set with brightness
    execute_command("AT+LED", "AT+LED", &mut led);         // Execute (current state)
    execute_command("AT+LED=0", "AT+LED", &mut led);       // Set (turn off)

    // 4. RESET command example
    println!("\n--- RESET Command ---");
    execute_command("AT+RST=?", "AT+RST", &mut reset);     // Test
    execute_command("AT+RST", "AT+RST", &mut reset);       // Execute

    // 5. Error handling examples
    println!("\n--- Error Handling ---");
    execute_command("AT+ECHO=2", "AT+ECHO", &mut echo);    // Invalid argument
    execute_command("AT+INFO=1", "AT+INFO", &mut info);    // Set not supported for INFO

    println!("\n=== Example completed ===");
}
