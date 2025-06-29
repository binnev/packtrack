{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixpkgs-python.url = "github:cachix/nixpkgs-python";
  };

  outputs = {
    self,
    nixpkgs,
    nixpkgs-python,
  }: let
    # We use this to tell nix which system we are targeting. Otherwise it will
    # default to whatever system we are running, which reduces the
    # reproducibility somewhat
    system = "x86_64-linux";

    pythonVersion = "3.13.3";

    # This is equivalent to `import nixpkgs { system = "x86_64-linux" }`.
    pkgs = import nixpkgs {inherit system;};
    # Nix supports cross-platform builds, and nixpkgs contains packages for many
    # systems. Examples include:

    # "x86_64-linux" -> regular 64-bit Linux (most desktops/servers)
    # "aarch64-linux" -> ARM-based Linux (like Raspberry Pi or AWS Graviton)
    # "x86_64-darwin" -> macOS on Intel
    # "aarch64-darwin" -> macOS on Apple Silicon (M1/M2)

    # If you import nixpkgs without specifying system, you might get an error
    # — or Nix might default to your current system, which isn’t always what
    # you want in a flake.

    myPython = nixpkgs-python.packages.${system}.${pythonVersion};
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        rustup
        cargo-watch
        myPython
        uv
        git-cliff # changelog generator
        mask # markdown-based replacement for make
      ];
      shellHook = ''
        uv venv --quiet
        source .venv/bin/activate
        uv pip install -r pyproject.toml --quiet
        pre-commit install -f --hook-type pre-commit --hook-type commit-msg --hook-type pre-push > /dev/null
        echo "Welcome!"
      '';
    };
  };
}
