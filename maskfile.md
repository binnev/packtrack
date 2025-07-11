# Run project tasks
This file defines and describes tasks that can be run with `mask`. 

Mask is a modern markdown-based replacement for `make`. The code blocks defined in this file are runnable via the `mask` CLI (try `mask --help`), and the surrounding markdown serves as the documentation.

See the [mask docs](https://github.com/jacobdeichert/mask) for more information.

<!-- A heading defines the command's name -->
## test

<!-- A blockquote defines the command's description -->
> Run all the tests

<!-- A code block defines the script to be executed -->
```sh
cargo test
```

## docs

> Hot-reload the documentation with `mkdocs`

```sh
mkdocs serve 
```

### docs bump 

> Create a new version of the docs

```sh 
version=$(cz version --project)
echo "Project version is $version"
regex="^([0-9]+)\.([0-9]+)\.([0-9]+)$"
if [[ $version =~ $regex ]]; then
    major="${BASH_REMATCH[1]}"
    minor="${BASH_REMATCH[2]}"
    major_minor="$major.$minor"
    echo "Truncated to docs version $major_minor"
else
    echo "Invalid version string format"
fi
mike set-default latest --allow-undefined
mike deploy $major_minor latest --update-aliases --push
```

## bump

> Create a new version of the project, docs, and changelog 

```sh
git-cliff -o CHANGELOG.md --bump \
&& git add CHANGELOG.md \
&& cz bump \
&& mask docs bump
```

## release 
> Release the current version to crates.io

```sh 
mask release cleanup
cargo publish
```

### cleanup 
> Clean up build artifacts

```sh
rm -rf target/
```

## ask (prompt)
> Ask the user a yes/no question (useful for scripting)

Returns a non-zero exit code if the answer is not "y", 

```sh
read -r -p "$prompt [y/N]: " response
if [[ ! "$response" =~ ^[Yy]$ ]]; then
echo "Aborted."
exit 1
fi
```

## changelog 

> Update the changelog with `git-cliff`

```sh
git-cliff -o CHANGELOG.md
```

Positional arguments

These are defined beside the command name within (round_brackets). They are required arguments that must be supplied for the command to run. The argument name is injected into the script's scope as an environment variable.

Example:

## pos (file) (test_case)

> Positional args example

~~~bash
echo "Testing $test_case in $file"
~~~

Optional arguments are defined within [square_brackets].

Example:

## opt [test_file]

> Optional args example

~~~bash
if [[ -n "$test_file" ]]; then
    echo "Run tests in $test_file..."
else
    echo "Running all tests...."
fi
~~~