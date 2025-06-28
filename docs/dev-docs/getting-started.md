# Getting Started 

This tutorial will guide you through setting up and running the Python Template project.

=== "nix + direnv" 

    This project is intended to be used with [nix](https://nixos.org/) and [direnv](https://direnv.net/). If you have these tools set up, simply opening a terminal in the project folder will perform all the necessary setup steps for you, including installing dependencies and pre-commit hooks :) 

    ## Steps
    1. Clone the repository:
    ```sh
    git clone <repository-url>
    cd rust-template
    ```

=== "manual"
    If you are not using nix and direnv, follow these steps to setup the project

    ## Steps
    1. Clone the repository:
    ```sh
    git clone <repository-url>
    cd rust-template
    ```

    2. Create a virtual environment (for the python helper tools): 
    ```sh 
    uv venv
    source .venv/bin/activate
    ```
    
    2. Install python helper dependencies:
    ```sh
    uv pip install -r pyproject.toml --all-extras
    ```

    3. Install pre-commit hooks:
    ```sh
    pre-commit install
    ```

    4. Install rust dependencies: 
    ```sh 
    cargo build 
    ```