# Testing 

## Run all tests
Running all tests should be as simple as this:
```sh
cargo test
```

## Run specific tests
Run only tests which have "multiply" in the file or function name: 
```sh
cargo test multiply
```

## Watch tests
To automatically re-run tests while you are working on them, use 
[`pytest-watcher`](https://github.com/olzhasar/pytest-watcher) (`q` to quit):
```sh
cargo watch -x test
```

You can also watch specific tests using the `-k` flag: 
```sh 
cargo watch -x "test multiply"
```

By default, tests are re-run 0.5 seconds after the last file edit. To increase the delay, pass `--delay`: 
```sh 
cargo watch -x "test multiply" --delay 1
```

## Getting test coverage 
TODO