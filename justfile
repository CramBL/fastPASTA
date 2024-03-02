import 'scripts/unique-error-codes.just'

# Absolute path to the directory containing the utility recipes to invoke them from anywhere
## USAGE: `{{PRINT}} green "Hello world"`
PRINT := join(justfile_directory(), 'scripts/pretty_print.just')

[private]
@default:
    just --list

alias c := check
alias b := build
alias t := test
alias l := lint
alias fc := full-check

# Run Full checks and format
full-check: lint format check test

# Check if it compiles without compiling
check:
    cargo check

# Build the application
build *ARGS:
    cargo build {{ ARGS }}

# Run the tests
test *ARGS: && check-unique-error-codes
    cargo test {{ ARGS }}
    ./tests/regression/regression_tests.sh
    

# Lint the code
lint *ARGS:
    cargo clippy {{ ARGS }}

# Format the code
format *ARGS:
    cargo fmt {{ ARGS }}

# Build the documentation (use `--open` to open in the browser)
doc *ARGS:
    cargo doc {{ ARGS }}


# Publish to crates.io
publish:
    cargo publish

# Run the application (use `--` to pass arguments to the application)
run *ARGS:
    cargo run {{ ARGS }}

# Needs the rust toolchain
env:
    rustc --version
    cargo --version

# List the dependencies
deps:
    cargo tree

# Update the dependencies
update:
    cargo update

# Clean the `target` directory
clean:
    cargo clean