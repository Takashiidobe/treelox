# Treelox

An implementation of a tree-walking interpreter in Rust for the Lox Language.

## Testing

To test the language implementation, this repo uses [`insta`](https://insta.rs/) which does snapshot testing. Instead of having to write a test which checks for the output of every lox program file in the `test-files` directory (there are 270+ programs), there's a glob that reads every lox file and gives it to the language implementation to run. This means adding a new test is as simple as writing a new lox program, dropping it in `test-files` and running `cargo test`. Afterwards, you can review the snapshot of the test, verify that the output looks correct (with `cargo insta review`) and continue onto the next test. There's no fragile test code that breaks every time you break an old program, you can re-review old snapshots and confirm the new behavior is correct.
