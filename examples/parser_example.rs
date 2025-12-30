//! Example using the AtParser with proper type handling

use parser_rs::context::AtContext;
use parser_rs::parser::AtParser;
use parser_rs::{Args, AtError, AtResult};

/// Simple command module for testing
pub struct TestCommand {
    pub value: u32,
}

impl AtContext for TestCommand {
    fn exec(&self) -> AtResult<'static> {
        Ok("Test command executed")
    }

    fn query(&mut self) -> AtResult<'static> {
        // In a real scenario, you would format the value dynamically
        // For this example, we use static strings
        if self.value == 0 {
            Ok("0")
        } else if self.value < 10 {
            Ok("1-9")
        } else {
            Ok("10+")
        }
    }

    fn test(&mut self) -> AtResult<'static> {
        Ok("Test: 0-100")
    }

    fn set(&mut self, args: Args) -> AtResult<'static> {
        let val_str = args.get(0).ok_or(AtError::InvalidArgs)?;
        self.value = val_str.parse().map_err(|_| AtError::InvalidArgs)?;
        Ok("OK")
    }
}

fn main() {
    println!("=== AtParser Example ===\n");

    // Create command instances
    let mut cmd1 = TestCommand { value: 0 };
    let mut cmd2 = TestCommand { value: 5 };
    let mut cmd3 = TestCommand { value: 10 };

    // Create parser instance
    let mut parser = AtParser::new();
    
    // Register commands
    let commands: &mut [(&str, &mut TestCommand)] = &mut [
        ("AT+CMD1", &mut cmd1),
        ("AT+CMD2", &mut cmd2),
        ("AT+CMD3", &mut cmd3),
    ];

    parser.set_commands(commands);

    println!("Registered commands: AT+CMD1, AT+CMD2, AT+CMD3\n");

    // Test cases
    let test_cases = vec![
        ("AT+CMD1", "Execute CMD1"),
        ("AT+CMD1?", "Query CMD1 value"),
        ("AT+CMD1=?", "Test CMD1"),
        ("AT+CMD1=42", "Set CMD1 to 42"),
        ("AT+CMD1?", "Query CMD1 after set"),
        ("AT+CMD2", "Execute CMD2"),
        ("AT+CMD2?", "Query CMD2 value"),
        ("AT+CMD3=100", "Set CMD3 to 100"),
        ("AT+CMD3?", "Query CMD3"),
        ("AT+UNKNOWN", "Unknown command error"),
        ("AT+CMD1=abc", "Invalid argument error"),
    ];

    for (cmd, description) in test_cases {
        println!("Test: {}", description);
        println!("  Command: {}", cmd);
        match parser.execute(cmd) {
            Ok(response) => println!("  Response: {}", response),
            Err(AtError::UnknownCommand) => println!("  Error: Unknown command"),
            Err(AtError::NotSupported) => println!("  Error: Not supported"),
            Err(AtError::InvalidArgs) => println!("  Error: Invalid arguments"),
        }
        println!();
    }

    println!("=== Example completed ===");
}
