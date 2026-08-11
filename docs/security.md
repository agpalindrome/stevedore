# Keeping secrets safe

stevedore exists to move secret *values* out of a source store and into a sink store. **A secret value must never
leak** — never printed, logged, written to disk, or handed to another process where anything but that process can
read it. This document explains what that means and how the code enforces it.

## What counts as a secret

stevedore models each store explicitly and marks certain fields of each record
as secret **values** — the material this tool exists to protect. Everything a
store does not mark as secret is treated as ordinary metadata.

Which fields are secret is a property of each store and is listed in that
store's own documentation (for Dashlane, see [the Personal notes](dcli/personal.md);
for Proton Pass, [the `pass-cli` notes](pass-cli/)). The guarantees below apply to
every field marked secret, whatever the store.

## How a leak is prevented

### Secret values redact themselves

Every secret is held in a `SecretValue`, a type that **cannot print itself**.
Formatting one — in a log line, an error, a debug dump — yields `<redacted>`,
never the value. Reading the actual bytes requires calling `.expose()`
explicitly, which is deliberately easy to search the code for. `SecretValue`
also has **no way to be serialized**, so a secret can never be accidentally
written back out as part of a data structure. Writing to a sink of course has to
send the value somewhere, and that one place is described below.

### Values never touch the disk

stevedore holds secret values **in memory** for the life of a run and never
writes them to disk — no cache, no temporary file, no export. Where a source
offers a bulk "export the vault" feature, stevedore does not use it: an export is
a plaintext copy of every secret in a file, exactly the artifact this tool
refuses to create. Where a sink offers to read an item from a file, stevedore
does not write that file either. How each store is worked without one is covered
in that store's documentation.

That guarantee covers stevedore. The tool it hands a value to is a separate
program with its own habits, and whether *it* keeps a copy is a question each
store's own notes answer.

### A secret reaches a sink through a pipe, not a command line

The arguments a program is run with are **visible to other processes** on the
machine for as long as it runs. So stevedore never puts a secret in one. When it
creates an item, the item — secrets included — is written to the sink tool's
**standard input**, a private channel between the two processes. Arguments carry
only the things that are not secret: which command to run, and which vault to
write to.

This is also why stevedore **creates** items rather than editing them: the sink
tool's update command takes new field values as command-line arguments, and that
is not a channel a secret may travel through.

Serializing a secret to send it is the one operation the redaction above has to
allow, and it is confined to a single function that every written item passes
through — the counterpart of the parsing gateway below, and just as short to
read.

### Parser errors can't echo the input

When stevedore parses the data a source returns, a parsing
library will, on a type mismatch, put **the offending value into its error
message** (`invalid type: string "hunter2", …`). If that error were then logged,
the secret would leak — and the redaction above would not help, because the leak
happens while reading the raw text, before any `SecretValue` exists.

Three things close this:

1. **One gateway.** All parsing of source output goes through a single function
   (`from_json`). It is the only place allowed to hold a parser error, and it
   throws the message away, keeping only *which* field failed and the *position*
   of the failure — neither of which contains a value.
2. **Errors that structurally can't hold a value.** The resulting "couldn't
   parse" error stores the field name as a fixed constant, not free text. It is
   impossible, at compile time, to smuggle a runtime value into it.
3. **A test that keeps it honest.** Automated tests feed the parser a marked fake
   secret in malformed input and assert the marker never appears in the error —
   in either its normal or its debug form. If a future change reintroduces a
   value-carrying error, these tests fail.

The same reasoning runs the other way: if an item cannot be built to send to a
sink, the resulting error says only that, because a serializer's message can
quote the value it choked on.

### A failed write says nothing about what was written

When a sink's tool rejects an item, stevedore reports why the *command* failed —
the tool's own message and exit status — and never the item it was given. A test
feeds a marked fake secret to a command that fails and asserts the marker appears
in neither the error nor its debug form.

## What this does not cover

- **Metadata is not redacted.** Unless identified and modelled as a secret value, fields will be treated as metadata.
- **The store's own tool has its own behaviour.** stevedore never authenticates
  and never unlocks a vault; that is set up separately with each store. How that tool
  stores credentials, what it writes to disk, and what it does with the system
  clipboard, is outside stevedore's control — see that store's notes for what its
  tool is known to keep.
- **Diagnostics from a store's tool are surfaced.** When a store's tool fails,
  its own error text (for example "not logged in") is shown so you can act on it.
  This is that tool's diagnostic channel, not the vault contents.
- **A written secret lives in the sink.** Once an item is created, protecting it
  is that store's job. stevedore does not remove the original from the source.

## Verifying it yourself

The mechanisms above live in a few small places in the `stevedore-secrets`
library: the redacting type in `secret.rs`, the parsing gateway and error type in
`error.rs`, the process handling in `cli.rs`, and the leak-regression tests
alongside each. They are deliberately compact so the guarantee can be read end to
end.
