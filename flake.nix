# 🔍 What each part does
#     inputs: brings in the Nix packages you’ll use. We use:
#             nixpkgs: the official NixOS package set
#                 flake-utils: to generate shells for multiple systems (x86_64-linux, etc.)
#             outputs: declares what your flake provides. We're providing:
#             A default devShell with all tools you need for Rust + OpenSSL
#             nativeBuildInputs: the packages your Rust project needs to compile native crates like openssl-sys
#         shellHook: sets environment variables so the Rust compiler knows where to find OpenSSL headers and libs
{
  description = "Rust dev environment with OpenSSL and pkg-config";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        # Native build tools and libraries for Rust
        nativeBuildInputs = with pkgs; [
          pkg-config
          openssl.dev
          gcc
          rustup # You can use rustc/cargo if you want pinned versions
        ];
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = nativeBuildInputs;

          # This ensures openssl can be found automatically
          shellHook = ''
            export OPENSSL_DIR=${pkgs.openssl.dev}
            export PKG_CONFIG_PATH=${pkgs.openssl.dev}/lib/pkgconfig
          '';
        };
      }
    );
}
