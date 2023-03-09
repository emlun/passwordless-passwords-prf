passwordless-passwords-prf
===

PROOF-OF-CONCEPT WORK IN PROGRESS


Usage
---


```
$ cargo install trunk
$ trunk serve
$ $BROWSER http://localhost:8080
```

Use https://github.com/emlun/yubikey-manager/tree/experiments/fido-vault-prf to
set up the vault and generate passwords (for now, may be added here too
eventually):

```
$ poetry install
$ poetry run ykman fido vault register
$ poetry run ykman fido vault generate foo
$ poetry run ykman fido vault export
```

Then paste the output from the above `export` command into the "Import vault config" box.
