# TCP Reverse Proxy

A small Rust TCP reverse proxy used to experiment with basic load balancing.

The proxy listens on `127.0.0.1:8080` and forwards incoming connections to three
local backend servers in round-robin order:

- `localhost:3001`
- `localhost:3002`
- `localhost:3003`

Each backend can serve a simple static `index.html` from the included
`server1`, `server2`, and `server3` directories.

## Current Behavior

- Accepts TCP connections on `127.0.0.1:8080`
- Selects the next backend using a round-robin counter
- Opens a TCP connection to the selected backend
- Reads the first request chunk from the client
- Rewrites `Host: 127.0.0.1:8080` to the selected backend host
- Sends the request upstream
- Streams the upstream response back to the client

## Requirements

- Rust toolchain
- A simple static file server for the sample backend directories

Python's built-in HTTP server works well for local testing.

## Running Locally

Start the three sample backend servers in separate terminals:

```sh
python3 -m http.server 3001 --directory server1
python3 -m http.server 3002 --directory server2
python3 -m http.server 3003 --directory server3
```

Then start the proxy:

```sh
cargo run
```

Send requests to the proxy:

```sh
curl http://127.0.0.1:8080
```

Repeated requests should cycle through the backend responses:

```text
server 1
server 2
server 3
```

## Testing

Run the Rust test suite:

```sh
cargo test
```

The current tests cover round-robin backend selection and empty backend list
rejection.

## Project Structure

```text
.
├── Cargo.toml
├── src
│   ├── lib.rs
│   └── main.rs
├── server1
│   └── index.html
├── server2
│   └── index.html
└── server3
    └── index.html
```

## Limitations

This is currently a learning-oriented implementation, not a production reverse
proxy.

Known limitations:

- Listener address and backend addresses are hard-coded
- Backend health checks are not implemented
- Retries and failover are not implemented
- Only the first client read is forwarded before streaming the response
- HTTP parsing is done with string replacement instead of a protocol-aware parser
- The Host header rewrite only handles `Host: 127.0.0.1:8080`
- TLS support is not currently wired into the proxy flow

## Possible Next Steps

- Move listener and backend addresses into configuration
- Add unit tests for round-robin backend selection
- Add integration tests for proxy behavior
- Replace ad hoc HTTP request rewriting with structured HTTP handling
- Add backend health checks and failure handling
