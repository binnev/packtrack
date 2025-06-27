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
pytest -n 6
```

## docs

> Hot-reload the documentation with `mkdocs`

```sh
mkdocs serve 
```

### docs bump 

> Create a new version of the docs

```sh 
mike set-default latest --allow-undefined
mike deploy $(cz version --project) latest --update-aliases --push
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
> Release the current version to PyPI

```sh 
mask release cleanup
python -m build \
&& twine check dist/* \
&& twine upload --repository testpypi dist/* --verbose \
&& mask ask "Does the TestPyPI release look ok?" \
&& twine upload --repository pypi dist/* --verbose
mask release cleanup
```

### cleanup 
> Clean up build artifacts

```sh
rm -rf dist/
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