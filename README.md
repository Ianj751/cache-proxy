# cache-proxy

`cache-proxy` is a simple HTTP caching proxy server written in Rust. It intercepts HTTP requests, forwards them to an origin server, and caches the responses in Redis for improved performance on repeated requests.

## Installation

To build and run the proxy, you'll need:

- [Rust](https://www.rust-lang.org/tools/install)
- [Redis](https://redis.io/) running locally or remotely

Clone the repository and build the project:

```bash
git clone https://github.com/your-username/cache-proxy.git
cd cache-proxy
cargo build --release
```

## Usage

```bash
./target/release/cache-proxy --port <PORT> --origin <ORIGIN_URL>
```

### Arguments
```
Argument        Description
--port          Port on which the proxy will listen (defaults to :8080)
--origin        Origin URL to which requests are sent
```

### Example

```bash
cache-proxy --port 8080 --origin https://jsonplaceholder.typicode.com
```

This will start the proxy server on http://localhost:8080 and forward uncached requests to https://jsonplaceholder.typicode.com.

## How It Works

    1. The client sends an HTTP request to cache-proxy.

    2. cache-proxy checks Redis for a cached version of the response.

    3. If found, it returns the cached response immediately.

    4. If not found, it forwards the request to the origin server.

    5. It stores the response in Redis and returns it to the client.
