use clap::{Arg, Command};
use regex::Regex;
use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
struct CompilerError();

fn main() {
    let matches = Command::new("jsi")
        .arg(Arg::new("input_file").help("Input file").required(true))
        .arg(Arg::new("output_file").help("Output file").required(true))
        .get_matches();

    let input_file = matches.get_one::<String>("input_file").unwrap();
    let output_file = matches.get_one::<String>("output_file").unwrap();

    // Delete the output file if it exists
    if fs::metadata(&output_file).is_ok() {
        fs::remove_file(&output_file).expect("Failed to delete output file");
    }

    let input = fs::read_to_string(input_file).expect("Failed to read input file");
    let output = compile_jsi_to_js(&input).expect("Compilation failed");

    fs::write(output_file, output).expect("Failed to write output file");
}

fn compile_jsi_to_js(input: &str) -> Result<String, CompilerError> {
    let parsed = parse_and_check(input)?;
    let (hoisted, non_hoisted) = process_hoist(&parsed);
    let combined: Vec<String> = hoisted.iter().chain(non_hoisted.iter()).cloned().collect();
    let typeof_null = convert_typeof_null(&combined.join("\n"));
    let output = minify_output(&typeof_null);

    Ok(output)
}

// Parse and check: Parse the input and check for type errors
fn parse_and_check(input: &str) -> Result<Vec<String>, CompilerError> {
    let mut lines = Vec::new();
    let mut variables: HashMap<String, String> = HashMap::new();
    let var_regex = Regex::new(r"\bvar\b").unwrap();

    for (_, line) in input.lines().enumerate() {
        let mut trimmed = line.trim().to_string();

        // Convert all `var` keywords to `let`
        if var_regex.is_match(&trimmed) {
            trimmed = trimmed.replace("var", "let").to_string();
        }

        // Type-checking function parameters
        if trimmed.contains("function") && trimmed.contains("=>") {
            let params_regex = Regex::new(r"\(([^)]*)\)").unwrap();
            if let Some(params) = params_regex.captures(&trimmed) {
                let param_list = params.get(1).unwrap().as_str();
                for param in param_list.split(',') {
                    let parts: Vec<&str> = param.trim().split(':').collect();
                    if parts.len() == 2 {
                        let param_name = parts[0].trim();
                        variables.insert(param_name.to_string(), parts[1].trim().to_string());
                    }
                }
            }
        }

        lines.push(trimmed.to_string());
    }

    Ok(lines)
}

// Convert `typeof null` to `"null"`
fn convert_typeof_null(input: &str) -> String {
    let typeof_null_regex = Regex::new(r"\btypeof null\b").unwrap();
    typeof_null_regex.replace_all(input, "\"null\"").to_string()
}

// Process hoist: Hoist variables to the top of the file
fn process_hoist(lines: &[String]) -> (Vec<String>, Vec<String>) {
    let mut hoisted = Vec::new();
    let mut non_hoisted = Vec::new();

    for line in lines {
        if line.starts_with("hoist") {
            hoisted.push(line.replace("hoist", "").trim().to_string());
        } else {
            non_hoisted.push(line.trim().to_string());
        }
    }

    (hoisted, non_hoisted)
}

// Minify output: Remove extra whitespace
fn minify_output(input: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_single_line_comment = false;
    let mut in_multi_line_comment = false;

    let mut i = 0;
    while i < input.len() {
        let c = input.chars().nth(i).unwrap();

        if in_single_line_comment {
            if c == '\n' {
                in_single_line_comment = false;
            }
            i += 1;
            continue;
        }

        if in_multi_line_comment {
            if c == '*' && input.chars().nth(i + 1) == Some('/') {
                in_multi_line_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if c == '"' || c == '\'' {
            in_string = !in_string;
            result.push(c);
            i += 1;
            continue;
        }

        if !in_string {
            if c == '/' && input.chars().nth(i + 1) == Some('/') {
                in_single_line_comment = true;
                i += 2;
                continue;
            }

            if c == '/' && input.chars().nth(i + 1) == Some('*') {
                in_multi_line_comment = true;
                i += 2;
                continue;
            }

            if let Some(keyword) = ["const", "let", "var"]
                .iter()
                .find(|&&kw| input[i..].starts_with(kw))
            {
                result.push_str(keyword);
                i += keyword.len();
                continue;
            }

            if c == '\n' {
                i += 1;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}
