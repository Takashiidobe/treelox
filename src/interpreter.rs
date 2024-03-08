use std::{fs::read_to_string, io, process};

use crate::scanner::Scanner;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Interpreter {
    args: Vec<String>,
}

impl Interpreter {
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    pub fn execute(&mut self) {
        match self.args.len() {
            1 => self.run_prompt(),
            2 => self.run_file(self.args[1].to_string()).unwrap(),
            _ => {
                eprintln!("Usage: treelox [path]");
                process::exit(64);
            }
        }
    }

    fn run_prompt(&mut self) {
        let mut line = String::new();

        loop {
            print!("> ");

            if let Err(e) = io::stdin().read_line(&mut line) {
                eprintln!("{}", e);
                break;
            }

            self.run(line.clone());
        }
    }

    fn run_file(&mut self, path: String) -> Result<(), io::Error> {
        let source = read_to_string(path)?;
        let scanner = self.run(source);
        if scanner.errors.had_error {
            process::exit(65);
        }
        Ok(())
    }

    fn run(&mut self, source: String) -> Scanner {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        for token in tokens {
            println!("{:?}", token);
        }
        scanner
    }
}

#[derive(Default, Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Errors {
    pub had_error: bool,
}

impl Errors {
    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    pub fn report(&mut self, line: usize, loc: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, loc, message);
        self.had_error = true;
    }
}