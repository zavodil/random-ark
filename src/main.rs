use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

#[derive(Deserialize, Serialize)]
struct Input {
    min: u32,
    max: u32,
}

#[derive(Serialize)]
struct Output {
    random_number: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read input from stdin
    let mut input_string = String::new();
    io::stdin().read_to_string(&mut input_string)?;

    // Parse input JSON
    let input: Input = serde_json::from_str(&input_string)?;

    // Generate random number using rand crate
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Calculate result in range
    let result = if input.max > input.min {
        rng.gen_range(input.min..=input.max)
    } else {
        input.min
    };

    // Create output
    let output = Output {
        random_number: result,
    };

    // Serialize to JSON and print to stdout
    let json = serde_json::to_string(&output)?;
    print!("{}", json);
    io::stdout().flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_parsing() {
        let input = Input { min: 1, max: 10 };
        let json = serde_json::to_string(&input).unwrap();
        let parsed: Input = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.min, 1);
        assert_eq!(parsed.max, 10);
    }

    #[test]
    fn test_output_serialization() {
        let output = Output { random_number: 42 };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("42"));
    }
}
