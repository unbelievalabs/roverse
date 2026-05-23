# Roverse

<p align="center">
  <a href="https://github.com/unbelievalabs/roverse/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/unbelievalabs/roverse?style=flat-square&color=4a92e1">
  </a>
  <a href="https://github.com/unbelievalabs/roverse/issues">
    <img src="https://img.shields.io/github/issues/unbelievalabs/roverse?style=flat-square&color=4a92e1">
  </a>
  <a href="https://discord.gg/T8FFRXDyMn">
    <img src="https://discordapp.com/api/guilds/1454939720172703807/widget.png?style=shield" alt="Discord Server">
  </a>
</p>

A lightweight proxy for Roblox API endpoints.

# Usage Guide

You need to have [Rust](https://www.rust-lang.org/tools/install) installed to build & run Roverse.

Compile and run the proxy:

```
# Clone the repository
git clone https://github.com/unbelievalabs/roverse.git
cd roverse

# Development
cargo run

# Production
cargo run --release
```

To use the proxy, convert any Roblox API URL by moving the subdomain into the first path segment:

```
Roblox URL:  https://{subdomain}.roblox.com/{path}
Roverse URL: http://127.0.0.1:8080/{subdomain}/{path}
```

## Address

By default, Roverse listens on `http://127.0.0.1:8080`. You can change this by setting the `ROVERSE_ADDR` environment variable.

## Security

To secure your proxy from unautharized access, set the `ROVERSE_SECRET` environment variable to require a `X-Proxy-Secret` header in all requests.

# License

This project is licensed under the MIT license. See [LICENSE](LICENSE) for more details.
