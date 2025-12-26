use std::io::{BufRead, Write};

pub mod introductory;

/// Fast input reader for competitive programming
pub struct Scanner {
    reader: Box<dyn BufRead>,
}

impl Scanner {
    pub fn new(reader: impl BufRead + 'static) -> Self {
        Self {
            reader: Box::new(reader),
        }
    }

    pub fn next_line(&mut self) -> String {
        let mut input = String::new();
        self.reader.read_line(&mut input).expect("Failed read");
        input.trim().to_string()
    }
}

/// Fast output writer for competitive programming (writes to memory buffer)
pub struct Writer(Vec<u8>);

impl Writer {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn print<T: std::fmt::Display>(&mut self, value: T) {
        write!(self.0, "{}", value).expect("Failed write");
    }

    pub fn println<T: std::fmt::Display>(&mut self, value: T) {
        writeln!(self.0, "{}", value).expect("Failed write");
    }

    pub(crate) fn into_string(self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.0)
    }
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

/// Test utilities for running and verifying test cases
pub mod testing {
    use super::*;
    use std::fs;
    use std::io::Cursor;
    use std::path::PathBuf;
    use std::time::Instant;

    /// Discover all test case numbers for a given problem
    pub fn discover_tests(category: &str, problem_name: &str) -> Vec<usize> {
        let test_dir = PathBuf::from("data").join(category).join(problem_name);

        let mut test_numbers = Vec::new();
        if let Ok(entries) = fs::read_dir(&test_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".in") {
                        if let Some(num_str) = file_name.strip_suffix(".in") {
                            if let Ok(num) = num_str.parse::<usize>() {
                                test_numbers.push(num);
                            }
                        }
                    }
                }
            }
        }

        test_numbers.sort_unstable();
        test_numbers
    }

    /// Run a single test case and return (expected, actual, duration) output
    pub fn run_test_case<F>(
        category: &str,
        problem_name: &str,
        test_num: usize,
        solve_fn: F,
    ) -> Result<(String, String, std::time::Duration), String>
    where
        F: FnOnce(&mut Scanner, &mut Writer),
    {
        let test_dir = PathBuf::from("data").join(category).join(problem_name);
        let in_file = test_dir.join(format!("{}.in", test_num));
        let out_file = test_dir.join(format!("{}.out", test_num));

        // Read input and expected output
        let input = fs::read_to_string(&in_file)
            .map_err(|e| format!("Failed to read {}: {}", in_file.display(), e))?;
        let expected = fs::read_to_string(&out_file)
            .map_err(|e| format!("Failed to read {}: {}", out_file.display(), e))?;

        // Run the solution and time it
        let input_reader = Cursor::new(input);
        let mut scanner = Scanner::new(input_reader);
        let mut writer = Writer::new();

        let start = Instant::now();
        solve_fn(&mut scanner, &mut writer);
        let duration = start.elapsed();

        let actual = writer
            .into_string()
            .map_err(|e| format!("Output is not valid UTF-8: {}", e))?;

        Ok((
            expected.trim().to_string(),
            actual.trim().to_string(),
            duration,
        ))
    }

    /// Verify all test cases for a problem
    pub fn verify_all_tests<F>(category: &str, problem_name: &str, solve_fn: F)
    where
        F: Fn(&mut Scanner, &mut Writer),
    {
        let test_cases = discover_tests(category, problem_name);
        assert!(
            !test_cases.is_empty(),
            "No test cases found for {}/{}",
            category,
            problem_name
        );

        let mut total_duration = std::time::Duration::ZERO;

        for test_num in test_cases {
            let result = run_test_case(category, problem_name, test_num, &solve_fn);
            match result {
                Ok((expected, actual, duration)) => {
                    total_duration += duration;
                    let secs = duration.as_secs_f64();
                    assert_eq!(
                        actual, expected,
                        "Test case {} failed (took {:.2}s)\nExpected:\n{}\nActual:\n{}",
                        test_num, secs, expected, actual
                    );
                }
                Err(e) => panic!("Test case {} error: {}", test_num, e),
            }
        }

        let total_secs = total_duration.as_secs_f64();
        println!("Total time: {:.2}s", total_secs);
    }

    /// Run all test cases and print results (for CLI usage)
    pub fn run_all_tests<F>(category: &str, problem_name: &str, solve_fn: F)
    where
        F: Fn(&mut Scanner, &mut Writer),
    {
        let test_cases = discover_tests(category, problem_name);
        if test_cases.is_empty() {
            println!("No test cases found for {}/{}", category, problem_name);
            return;
        }

        println!(
            "Running {} test cases for {}/{}...",
            test_cases.len(),
            category,
            problem_name
        );

        let mut passed = 0;
        let mut failed = 0;
        let mut total_duration = std::time::Duration::ZERO;

        for test_num in &test_cases {
            match run_test_case(category, problem_name, *test_num, &solve_fn) {
                Ok((expected, actual, duration)) => {
                    total_duration += duration;
                    let secs = duration.as_secs_f64();
                    if actual == expected {
                        println!("✓ Test case {}: PASSED ({:.2}s)", test_num, secs);
                        passed += 1;
                    } else {
                        println!("✗ Test case {}: FAILED ({:.2}s)", test_num, secs);
                        println!("  Expected: {}", expected);
                        println!("  Actual:   {}", actual);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("✗ Test case {}: ERROR - {}", test_num, e);
                    failed += 1;
                }
            }
        }

        let total_secs = total_duration.as_secs_f64();
        println!("\nResults: {} passed, {} failed", passed, failed);
        println!("Total time: {:.2}s", total_secs);
        if failed > 0 {
            std::process::exit(1);
        }
    }
}
