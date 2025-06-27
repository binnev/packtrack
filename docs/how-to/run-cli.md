# Running the CLI 

Run the CLI: 
```sh 
cargo run 
```

Show the help with 
```sh
cargo run -- -h
```

It prints: 

```
A CLI calculator application that performs basic arithmetic operations

Usage: rust-template [OPTIONS] <COMMAND>

Commands:
  add       Adds two numbers
  subtract  Subtracts the second number from the first
  multiply  Multiplies two numbers
  divide    Divides the first number by the second
  help      Print this message or the help of the given subcommand(s)

Options:
  -v, --verbosity...  Set verbosity. `-v` = 1, `-vvv` = 3
  -h, --help          Print help
  -V, --version       Print version
```

## Example 

```sh
cargo run add 420 69
```

result: 
```
Result: 489
```