A [WebTransport sample] provides a [Python webserver], intended to be connected
from the web client.

This Rust sample aims to reimplement the Python webserver (with the same
[logic]) in async Rust using the library [Wtransport].

[WebTransport sample]: https://github.com/GoogleChrome/samples/tree/gh-pages/webtransport
[Python webserver]: https://github.com/GoogleChrome/samples/blob/gh-pages/webtransport/webtransport_server.py
[WTransport]: https://github.com/BiagioFesta/wtransport
[logic]: https://github.com/GoogleChrome/samples/blob/2bb27d915e3cbfe5ba4fc80fe4922baca16db703/webtransport/webtransport_server.py#L95-L103

## Run

Generate certificate as explained [here][instructions].

[instructions]: https://github.com/GoogleChrome/samples/blob/2bb27d915e3cbfe5ba4fc80fe4922baca16db703/webtransport/webtransport_server.py#L34-L75

Copy the certificate into the project root directory as `cert.pem` and `key.pem`.

Run the server:

```bash
cargo build --release
target/release/wtransport-sample
```

Run the client (adapt the [certificate fingerprint]):

[certificate fingerprint]: https://github.com/GoogleChrome/samples/blob/2bb27d915e3cbfe5ba4fc80fe4922baca16db703/webtransport/webtransport_server.py#L58C1-L63

```
chromium --origin-to-force-quic-on=localhost:4433 --ignore-certificate-errors-spki-list=rOxva4Y8FcAUzOje9N66vJTYLxhSK9r5t2tVVEe2bdE= client.html
```
