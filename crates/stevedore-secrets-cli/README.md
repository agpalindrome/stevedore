# stevedore-secrets-cli

The command-line interface for
[`stevedore`](https://github.com/agpalindrome/stevedore). Installs the
`stevedore` binary.

```console
$ stevedore stores
sources: dashlane
sinks:   proton-pass
```

`move` carries secrets from Dashlane into a Proton Pass vault. It reports what
would change and writes nothing until given `--apply`.

```console
$ stevedore move --to-vault "My Vault"
$ stevedore move --to-vault "My Vault" --apply
```

Licensed under either of Apache-2.0 or MIT at your option.
