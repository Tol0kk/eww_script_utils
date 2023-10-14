{
  description = "<PROGRAM>";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        buildInputs = with pkgs; [
          ## General
          pkg-config
          llvm
          (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
        ];
      in
      with pkgs;

      {
        defaultPackage = rustPlatform.buildRustPackage {
          pname = "<PROGRAM>";
          version = "1.0.0";
          src = ./.;
          cargoSha256 = "sha256-dPzvNG/kckEqdQraj+I94VH7XZPGtM2h1da+N8tbqA8=";
          nativeBuildInputs = [ rust wasm-bindgen-cli ];
          buildPhase = ''
            cargo build --release --target=wasm32-unknown-unknown

            echo 'Creating out dir...'
            mkdir -p $out/src;

            # Optional, of course
            # echo 'Copying package.json...'
            # cp ./package.json $out/;

            echo 'Generating node module...'
            wasm-bindgen \
              --target nodejs \
              --out-dir $out/src \
              target/wasm32-unknown-unknown/release/gcd.wasm;
          '';
          installPhase = "echo 'Skipping installPhase'";
        };

        devShells.default = mkShell {
          inherit buildInputs;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      }
    );
}
