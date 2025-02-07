# Pyro Warehouse 🏭

Pyro Warehouse is a flexible and extensible game server software distribution system that provides a unified API for downloading server binaries from various games and loaders. Currently, it only supports Minecraft server distributions. However, its architecture is completely game-agnostic and designed to accommodate any game server software.

## Getting Started

```bash
# Clone the repository
git clone https://github.com/pyrohost/warehouse.git

# Change to the project directory
cd warehouse

# Build the Rust project
cargo build --release # --release is optional but recommended

# Run the server
./target/release/warehouse # or `cargo run --release`
```

For detailed API documentation, please visit the `/docs/` endpoint when running the server. (e.g. `http://localhost:8080/docs/`)

## Configuration

Pyro Warehouse is configured using environment variables. The following variables are available:

| Variable | Description | Default |
|----------|-------------|---------|
| `WAREHOUSE_BIND_ADDRESS` | The address to bind the server to | `127.0.0.1:8080` |
| `WAREHOUSE_STORAGE_PATH` | The path to cache server binaries | `./storage` |
| `WAREHOUSE_LOG_LEVEL` | Logging level (error, warn, info, debug, trace) | `info` |
| `WAREHOUSE_CACHE_TTL` | Cache time-to-live in seconds | `3600` |

These variables can also be set in a `.env` file in the runtime directory.

## Credits

This application is maintained by Pyro Inc., while we do not provide direct support for this software, we welcome contributions, bug reports, and feature requests. Get community support on our [Discord server](https://discord.gg/pyrohost)!

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
