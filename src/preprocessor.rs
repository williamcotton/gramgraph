use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

pub fn expand_variables(input: &str, variables: &HashMap<String, String>) -> Result<String> {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '$' => {
                // Found potential variable
                let var_name = consume_identifier(&mut chars);
                if var_name.is_empty() {
                    // Just a lone $, treat as literal
                    output.push('$');
                } else {
                    // Look up variable
                    if let Some(val) = variables.get(&var_name) {
                        output.push_str(val);
                    } else {
                        return Err(anyhow!("Variable '${}' not defined", var_name));
                    }
                }
            }
            _ => output.push(c),
        }
    }

    Ok(output)
}

fn consume_identifier(chars: &mut Peekable<Chars>) -> String {
    let mut name = String::new();
    // Identifiers start with alpha or _
    if let Some(&c) = chars.peek() {
        if !c.is_alphabetic() && c != '_' {
            return name;
        }
    }

    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' {
            name.push(c);
            chars.next();
        } else {
            break;
        }
    }
    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expansion() {
        let mut vars = HashMap::new();
        vars.insert("col".to_string(), "height".to_string());
        vars.insert("val".to_string(), "10".to_string());

        let input = "aes(x: $col) | point(size: $val)";
        let output = expand_variables(input, &vars).unwrap();
        assert_eq!(output, "aes(x: height) | point(size: 10)");
    }

    #[test]
    fn test_string_interpolation() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), "date".to_string());

        // $x inside quotes should now be expanded
        let input = "aes(x: $x) | labs(title: \"Value in $x\")";
        let output = expand_variables(input, &vars).unwrap();
        assert_eq!(output, "aes(x: date) | labs(title: \"Value in date\")");
    }

    #[test]
    fn test_lone_dollar() {
        let vars = HashMap::new();
        let input = "Cost ($)";
        let output = expand_variables(input, &vars).unwrap();
        assert_eq!(output, "Cost ($)");
    }

    #[test]
    fn test_undefined_variable() {
        let vars = HashMap::new();
        let input = "aes(x: $missing)";
        let result = expand_variables(input, &vars);
        assert!(result.is_err());
    }
}
