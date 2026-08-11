# stevedore

A stevedore moves cargo between vessels; `stevedore` moves **secrets** between
stores. It's a Rust workspace with three crates: a **library**, a **CLI**, and
an **MCP server**.

## Stores

🟢 working · 🟡 in progress · 🔵 planned · 🔴 not planned

| Store                             | Read | Write |
| --------------------------------- | :--: | :---: |
| [Dashlane](docs/dcli/)            |  🟢  |  🔴   |
| [Proton Pass](docs/pass-cli/)     |  🔵  |  🟢   |

## Moving secrets

Both stores' own CLIs must already be installed and logged in — stevedore never
authenticates. See [`docs/dcli/`](docs/dcli/) and
[`docs/pass-cli/`](docs/pass-cli/).

```console
# what would change — writes nothing:
stevedore move --to-vault "My Vault"

# carry it out:
stevedore move --to-vault "My Vault" --apply
```

A move creates items; it never updates or deletes one. It passes over anything
already in the vault under the same title and kind, so a move that is repeated,
or resumed after an interruption, does not leave duplicates.

Two things it reports rather than decides: logins whose Dashlane note text has
nowhere to go in Proton Pass, and items it found in the vault but could not read
the state of. Both are listed by title so you can settle them yourself.

## What it is (and isn't)

- **A mover.** stevedore _migrates_ secret values from a source store to a sink
  store.
- **Not a resolver.** Runtime secret _resolution_ — an app fetching its own key
  at start-up — is what [secretspec](https://github.com/cachix/secretspec) does.
- **Safe by default.** Secret values redact themselves in logs by construction —
  passwords, note contents, 2FA seeds and attachment keys alike — and nothing is
  ever exported to disk. See [Keeping secrets safe](docs/security.md) for how
  that guarantee is enforced.

## Layout

| Crate                                                              | What it is               |
| ------------------------------------------------------------------ | ------------------------ |
| [`crates/stevedore-secrets`](crates/stevedore-secrets)             | The core library.        |
| [`crates/stevedore-secrets-cli`](crates/stevedore-secrets-cli)     | The command-line binary. |
| [`crates/stevedore-secrets-mcp`](crates/stevedore-secrets-mcp)     | The MCP server.          |

## Develop

The dev environment is a Nix flake; [direnv](https://direnv.net/) loads it on
entry.

```console
# one-time, per clone:
cp .envrc.example .envrc && direnv allow   # loads the flake dev shell
git config core.hooksPath .githooks        # arm the git hooks (pre-commit secret scan + pre-push fmt/clippy)

# or without direnv:
nix develop

# then, the usual loop:
cargo test
cargo clippy --all-targets
cargo run -p stevedore-secrets-cli -- stores
```

CI runs `fmt`, `clippy`, `test`, the doc tests, `cargo doc --no-deps`, and
`cargo deny check` through the same flake. Lints carry their level in
`Cargo.toml`, so plain `cargo clippy` gives the verdict CI does — see
[`CONTRIBUTING.md`](CONTRIBUTING.md) for the full loop.

### A note on secrets

stevedore's inputs are _your other stores' credentials_. They are supplied
deliberately for a single run — never committed, never loaded ambiently into
every shell. The `.gitignore` and the `.githooks/pre-commit` guard exist to keep
it that way.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at
your option.
