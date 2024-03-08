use std::env;
use treelox::interpreter::Interpreter;

fn main() {
    let args: Vec<_> = env::args().collect();

    let mut interpreter = Interpreter::new(args);
    interpreter.execute();
}
