# Rust Reverse Proxy Server

This project implements a Rust-based reverse proxy server that forwards the incoming requests to an origin server and caches the response data. The server leverages an in-memory caching mechanism to optimize performance by saving and reusing the response data for a period of 30 seconds (Time To Live - TTL).

## Getting Started

These instructions will get you a copy of the project up and running on your local machine.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) programming language.

### Installing and Running

1. **Clone the repository**

```bash
git clone https://github.com/Samuel3Shin/rust-reverse-proxy-server.git
```

2. **Navigate to the directory**

```bash
cd rust-reverse-proxy-server
```

3. **Build the project**

```bash
cargo build
```

4. **Run the server**

```bash
cargo run
```

The server will start and listen on the IP address and port specified in the `Settings.toml` file.

### Testing

To test the application, use a web browser and enter the URL in the following format: `http://127.0.0.1:7878/{your_desired_url}`.

Replace `{your_desired_url}` with the actual URL you want to request. The server will forward your request to the origin server specified in `{your_desired_url}`.

Example URLs:

- http://127.0.0.1:7878/https://blockstream.info/api/blocks/0
- http://127.0.0.1:7878/https://blockstream.info/api/blocks/1
- http://127.0.0.1:7878/https://blockstream.info/api/blocks/101

These URLs will send requests to `https://blockstream.info/api/blocks/{block_number}` via the reverse proxy, and the response from the origin server will be returned to your browser.

## Configuration

The server's settings can be adjusted in the `Settings.toml` file. You can configure the following parameters:

- `LOCAL_HOST_IP`: The IP address where your server will run.
- `LOCAL_HOST_PORT`: The port your server will listen on.
- `REMOVE_OLD_CACHE_INTERVAL`: The frequency (in seconds) to check and remove expired cache data.
- `CACHE_LIFETIME`: The lifetime (in seconds) of cache data.