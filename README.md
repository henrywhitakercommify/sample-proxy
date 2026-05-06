# sample-proxy

A minimal HTTP reverse proxy in Rust. Forwards every incoming request to a configured origin, preserving method, path+query, headers, and body. The `Host` header is rewritten to match the origin.

## Configuration

| Env var      | Required | Description                                 |
| ------------ | -------- | ------------------------------------------- |
| `ORIGIN_URL` | yes      | Upstream base URL, e.g. `https://api.example.com` |
| `RUST_LOG`   | no       | `tracing-subscriber` filter (default `info`) |

## Run

```sh
ORIGIN_URL=https://httpbin.org cargo run
```

Listens on `0.0.0.0:8080`. Logs are emitted as JSON.

```sh
curl -s http://localhost:8080/get
```

## Behaviour

- Any method, any path — a single fallback handler proxies everything.
- Request body is fully buffered before forwarding.
- Response body is streamed back to the client.
- Returns `502` on upstream connection errors, `400` on body read errors.
