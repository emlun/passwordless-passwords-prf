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


Building
---


Build static web assets to `dist/`:

```sh
$ export RP_ID="example.org"
$ export RP_NAME="Passwordless Passwords demo"
$ trunk build --release
```

Set the `RP_ID` environment variable to the domain where the app will be hosted.
The `RP_NAME` value may be shown to users during credential registration.

Build a Docker image, run the following after the above:

```sh
$ docker build .
```
