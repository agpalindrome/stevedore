# stevedore-secrets

The core library behind [`stevedore`](https://github.com/ojhermann-org/stevedore).

## What it offers

- `SecretValue` — a secret that redacts itself in `Debug` and `Display`. Read
  it deliberately with `.expose()`.
- `dashlane` — reads a Dashlane vault's logins, secure notes and secrets through
  Dashlane's own `dcli`, keeping values in-process and access read-only.
- `proton` — creates logins and secure notes in a Proton Pass vault through
  Proton's own `pass-cli`, passing values on stdin rather than a command line.
- `mover` — plans a move from the one into the other, then carries it out. A
  plan writes nothing, so it is safe against any vault.

Licensed under either of Apache-2.0 or MIT at your option.
