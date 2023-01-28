let
  sources = import ./nix/sources.nix;
  rust-overlay = import sources.rust-overlay;
  nixpkgs = import sources.nixpkgs {
    overlays = [rust-overlay];
  };
  rust-toolchain-version = (nixpkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain).override {
    extensions = [ "rust-analyzer" "rust-src" ];
  };
in
  nixpkgs.mkShell {
    name = "auto-dev";
    nativeBuildInputs = with nixpkgs; [
      # Required for openssl-sys crate
      openssl
      pkg-config

      # Rust core
      rust-toolchain-version
      # Neat helper tools
      cargo-audit
      cargo-edit
      cargo-flamegraph
      cargo-show-asm

      # Nix tools
      niv
    ];

    # Always enable rust backtraces in development shell
    RUST_BACKTRACE = "1";
  }
