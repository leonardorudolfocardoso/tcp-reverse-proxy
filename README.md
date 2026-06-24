# TCP Reverse Proxy

A small Rust TCP reverse proxy used to experiment with basic load balancing.

The proxy listens on `127.0.0.1:8080` and forwards incoming connections to three
local backend servers in round-robin order:

- `localhost:3001`
- `localhost:3002`
- `localhost:3003`

Each backend can serve a simple static `index.html` from the included
`examples/static-backends/server1`, `examples/static-backends/server2`, and
`examples/static-backends/server3` directories.

## Current Behavior

- Accepts TCP connections on `127.0.0.1:8080`
- Selects the next healthy backend using a round-robin counter
- Opens a TCP connection to the selected backend
- Marks a backend unhealthy when the proxy cannot connect to it
- Skips unhealthy backends while routing new connections
- Checks unhealthy backends every 5 seconds with a TCP connection attempt
- Restores reachable backends to the rotation
- Reads the first request chunk from the client
- Rewrites `Host: 127.0.0.1:8080` to the selected backend host
- Sends the request upstream
- Streams the upstream response back to the client

## Requirements

- Rust toolchain
- A simple static file server for the sample backend directories

Python's built-in HTTP server works well for local testing.

## Running Locally

Start the three sample backend servers:

```sh
make example
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

The current tests cover round-robin backend selection, empty backend list
rejection, unhealthy backend skipping, health restoration, and unhealthy backend
address listing.

## Project Structure

```text
.
├── Cargo.toml
├── Makefile
├── src
│   ├── lib.rs
│   └── main.rs
└── examples
    └── static-backends
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
- Health checks only verify that a TCP connection can be established; they do
  not validate HTTP response status or content
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
- Add richer health checks and retry/failover behavior
