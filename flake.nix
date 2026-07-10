{
  # Based on https://github.com/the-nix-way/dev-templates/blob/4eab4b7c62077d14773edc6abcb4bf7664bdcc1f/rust/flake.nix
  description = "Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1"; # tracks nixpkgs-unstable
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "https://flakehub.com/f/nix-community/fenix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    {
      overlays.default = final: prev: {
        rustToolchain = fenix.packages.${prev.stdenv.hostPlatform.system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-OATSZm98Es5kIFuqaba+UvkQtFsVgJEBMmS+t6od5/U=";
        };
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };
      in
      {
        formatter = pkgs.nixfmt;
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            cargo-expand
            nixfmt
            statix
            pre-commit
          ];
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
          # Required at test/runtime: openssl-sys links libssl.so.3 dynamically.
          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.openssl.out}/lib:''${LD_LIBRARY_PATH:-}
          '';
        };
      }
    );
}
