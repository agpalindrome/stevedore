# Contributing to stevedore

Thanks for your interest. stevedore is early — an honest scaffold with one route
(Dashlane → Proton Pass) under construction — so the most useful contributions
right now are around that first route and the core library shape.

## Ground rules

- **Never commit a secret.** This tool handles secret values; a stray export or a
  pasted credential must never land in git. The `.githooks/pre-commit` guard
  scans staged changes with `gitleaks` and blocks secret files — arm it with
  `git config core.hooksPath .githooks`. Don't work around it.
- **Read [`CLAUDE.md`](CLAUDE.md) first.** It records the conventions and why
  things are the way they are — mover-not-resolver; no `Store` trait until store
  #3/#4; MCP deferred.

## Development

The dev environment is a Nix flake, loaded by direnv (or `nix develop`):

```console
cp .envrc.example .envrc && direnv allow
cargo fmt
cargo clippy --all-targets
cargo test
cargo doc --no-deps
cargo deny check
```

CI runs exactly these through the same flake, so green locally means green in CI.

Lints carry their real level in `Cargo.toml`, so plain `cargo clippy` gives the
same verdict CI does — no `-D warnings` needed. `cargo doc` is a gate, not a
convenience: it is the only command that enforces the rustdoc lints. Suppress a
lint with `#[expect(…, reason = "…")]` rather than `#[allow]`, so the
suppression fails once it stops applying.

Run these **inside the dev shell**. `rustfmt.toml` uses options that need a
nightly rustfmt, which the flake pins and supplies — so `cargo fmt` is right as
written, with no `+nightly`. A stable rustfmt silently ignores those options and
reports the tree as clean either way, so `cargo fmt --check` outside the shell
proves nothing.

## Landing a change

- Work on a branch and open a PR — `main` requires a PR and the `ci` check.
- Keep the change and its docs together: if you change behaviour, the public API,
  or a design decision, update the affected README / rustdoc / `--help` in the
  **same** PR.
- Squash-merge; delete the branch after.

## License

By contributing you agree that your contributions are licensed under the same
terms as the project: Apache-2.0 or MIT, at the user's option.
