# Proton Pass CLI (`pass-cli`)

stevedore's Proton Pass **sink** writes into a vault through Proton's
`pass-cli`, handing secret values to it **on stdin** — never as command-line
arguments, which other processes on the machine can read.

**stevedore never authenticates.** Logging in is a **one-time setup the user
performs with `pass-cli` directly**. stevedore assumes a working session.

## Install

Install `pass-cli` by following Proton's own
[installation guide](https://proton.me/support/pass-cli).

## Log in

```console
pass-cli login          # opens a browser
pass-cli test           # "Connection successful"
```

## How stevedore writes to a vault

stevedore creates items in a vault you name, by running `pass-cli` and passing
each item as JSON on the tool's standard input. Nothing is written to disk, and
no secret ever appears in a command line.

## What `pass-cli` keeps on disk

`pass-cli` does not cache item content. Each command fetches what it needs from
Proton, so the secrets stevedore writes are not left behind in a local copy.

What it does keep — under `~/.local/share/proton-pass-cli/.session/` on Linux —
is a database of vault keys and settings, and a record of the logged-in session.
Both are encrypted, under a key held in your operating system's keyring.

One caveat, for a database created before `pass-cli` 2.2.4: that file was made
world-readable, and only the owner-only directory around it kept it private.
2.2.4 creates it `0600`, but only when the file is absent, so an older one keeps
its mode indefinitely — `chmod 600` on it is harmless. Its contents are
encrypted, so this matters if the file is lifted out of that directory, by a
backup or sync tool or a container mount. Observed on Linux, 2026-08-05.

## On Linux, a reboot ends the session

By default `pass-cli` keeps the key to its local database in the Linux kernel
keyring, which a reboot clears. The next `pass-cli` command after a restart does
not fail — it deletes the local session and asks you to log in again, even a
read-only one:

```console
$ pass-cli vault list
Error: Local encryption key not found but local data exists. Forcing logout for security.
```

No vault content is lost, since none is held locally. What goes is the session,
and `pass-cli login` is interactive — so an unattended move cannot recover on
its own. **Log in again after a reboot, before running stevedore.**

To keep the session across reboots, set `PROTON_PASS_LINUX_KEYRING=dbus` and run
a Secret Service such as GNOME Keyring; `pass-cli` then stores the key there
instead. Changing the backend leaves the current session unreadable, so log in
once more after switching.

Observed on Linux, 2026-08-05, with `pass-cli` 2.2.4. The D-Bus route is
untested here, on a machine that runs no Secret Service.

## What can be written

🟢 supported · 🔵 planned · 🔴 not possible

| Proton Pass UI | `pass-cli` | stevedore |
| -------------- | :--------: | :-------: |
| Logins         |     🟢     |    🟢     |
| Secure notes   |     🟢     |    🟢     |
| Credit cards   |     🟢     |    🔵     |
| Identities     |     🟢     |    🔵     |
| SSH keys       |     🟢     |    🔵     |
| WiFi           |     🟢     |    🔵     |
| Custom items   |     🟢     |    🔵     |
| Aliases        |     🟢     |    🔵     |
| Attachments    |     🔴     |    🔴     |

`pass-cli` can download an attachment but not upload one.

### Logins

**Written:** title, username, email, password, a 2FA (TOTP) token, and website
addresses.

**Treated as secret:** the password and the 2FA (TOTP) token.

**Not written:** a note attached to the login. A Proton login carries one, but
`pass-cli`'s item template has no field for it — supplying one anyway is accepted
and silently discarded — and the only command that would set it takes the value
as a command-line argument.

### Secure notes

**Written:** title and content.

**Treated as secret:** the note's content.

## What can be listed

stevedore can ask a vault what it already holds: each item's title, kind, state
(live or trashed) and identifiers.

**Descriptions only, never contents.** `pass-cli` returns item values only when
asked with `--show-secrets`, and stevedore never asks — so listing a vault brings
no secret into the process. Reading secrets *out of* Proton Pass is a separate
thing stevedore does not do.

## Items are created, never changed

stevedore only creates items. It does not update or delete an existing one.

A move therefore checks the vault first, and passes over anything already there
under the same title and kind — so running the same move twice does not leave two
of everything, and a move interrupted part-way can simply be run again.

## Unknown fields are dropped in silence

`pass-cli` creates an item from a template carrying a field it does not know,
without the field and without a word. A name that does not match the template
loses data quietly, so the names stevedore sends are pinned by test to
`pass-cli item create <kind> --get-template`.
