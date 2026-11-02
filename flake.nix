{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, ... }@inputs:
    {
      overlays.default = final: _: {
        rustToolchain = final.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      };
    }
    // inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.default
            self.overlays.default
          ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            cargo-audit
            cargo-expand
            pre-commit
            openssl
            pkg-config
            self.formatter.${system}
          ];

          env = {
            # openssl-sys links libssl.so.3 dynamically; make it resolvable at test/runtime.
            LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [ openssl ];
          };
        };

        formatter = pkgs.nixfmt;
      }
    );
}
