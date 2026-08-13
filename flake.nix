{
  description = "stevedore — move secrets between password managers and vaults (library + CLI + MCP)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Pinned, not `stable.latest`: `pedantic` is denied, so a toolchain bump
        # can break the build with no source change. Moving this is a visible
        # one-line diff rather than a side effect of `nix flake update`. Keep it
        # in step with `rust-version` in Cargo.toml — that is the support floor
        # this promises consumers, this is the version it is built with.
        rustToolchain = pkgs.rust-bin.stable."1.97.1".default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];
        };

        # Five options in rustfmt.toml are nightly-only; a stable rustfmt warns
        # and ignores them. Pinned to a date rather than tracking nightly:
        # nightly rustfmt output carries no stability guarantee, so an unpinned
        # one would reformat the tree on its own schedule. Move it deliberately.
        nightlyRustfmt = pkgs.rust-bin.nightly."2026-07-01".rustfmt;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            # Ahead of rustToolchain, so a bare `rustfmt` is the nightly one too.
            nightlyRustfmt
            rustToolchain
            pkgs.cargo-deny # dependency license / advisory / ban policy (deny.toml)
            pkgs.bacon # background `cargo check`/clippy/test runner
            pkgs.gitleaks # secret scanner for the pre-commit guard (.githooks/pre-commit)
            pkgs.actionlint # GitHub Actions workflow linter (.github/workflows)
            pkgs.yamllint
            pkgs.vale # prose linter for the vendored house style (scripts/prose.sh)
          ];

          env.RUST_BACKTRACE = "1";

          # `cargo fmt` resolves rustfmt through this, so plain `cargo fmt` gets
          # the nightly one. There is no `cargo +nightly` here: the `+toolchain`
          # syntax is rustup's, and this shell has no rustup.
          env.RUSTFMT = "${nightlyRustfmt}/bin/rustfmt";

          shellHook = ''
            echo "stevedore dev shell — $(rustc --version)"
          '';
        };

        # `nix fmt` formats the Nix files in this repo.
        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
