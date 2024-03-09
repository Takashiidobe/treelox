use std::fs;
use std::io::{self, Write};
use std::process::exit;

use treelox::error::Error;
use treelox::interpreter::Interpreter;
use treelox::parser::Parser;
use treelox::scanner::Scanner;

struct Lox {
    interpreter: Interpreter,
}

enum Input {
    Repl,
    File,
}

impl Lox {
    fn new() -> Self {
        Lox {
            interpreter: Interpreter::default(),
        }
    }

    fn run_file(&mut self, path: &str) -> Result<(), Error> {
        let source = fs::read_to_string(path)?;
        self.run(source, Input::File)
    }

    fn run_prompt(&mut self) -> Result<(), Error> {
        let mut buffers = vec![];
        loop {
            let mut buffer = String::new();
            print!("> ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut buffer)?;
            if self.run(buffer.clone(), Input::Repl).is_ok() {
                buffers.push(buffer);
            }
        }
    }

    fn run(&mut self, source: String, input: Input) -> Result<(), Error> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        match input {
            Input::Repl => match parser.parse_exprs() {
                Ok(expressions) => {
                    self.interpreter.interpret_expressions(&expressions)?;
                }
                Err(_) => {
                    let statements = parser.parse()?;
                    self.interpreter.interpret(&statements)?;
                }
            },
            Input::File => {
                let statements = parser.parse()?;
                self.interpreter.interpret(&statements)?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    let mut lox = Lox::new();
    match &args[..] {
        [_, file] => match lox.run_file(file) {
            Ok(_) => (),
            Err(Error::Runtime { .. }) => exit(70),
            Err(Error::Parse { .. }) => exit(65),
            Err(Error::Io(_)) => unimplemented!(),
        },
        [_] => lox.run_prompt()?,
        _ => {
            eprintln!("Usage: treelox [script]");
            exit(64)
        }
    }
    Ok(())
}
