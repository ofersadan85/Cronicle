# Cronicle Rust Implementation

This is a Rust port of the Cronicle task scheduler, originally written in Node.js.

## Project Status

This is an **initial implementation** providing the foundational structure for a Rust-based Cronicle server. The following components have been implemented:

### ✅ Completed Components

- **Project Structure**: Cargo-based Rust project with proper dependency management
- **Configuration System**: JSON-based configuration loading compatible with original config.json format
- **Basic Web Server**: HTTP server using Axum framework
- **Engine Core**: Basic engine initialization and directory setup
- **Storage Abstraction**: Generic trait for multiple backends (filesystem implemented)
- **Scheduler**: Cron-like event scheduling with timezone support
- **Job Management**: Job lifecycle tracking (launch, complete, abort)
- **Health Check API**: `/health` endpoint for server status
- **Status API**: `/api/app/status` endpoint showing active jobs and server state
- **Tests**: 19 comprehensive unit tests

### 🚧 In Progress / Not Yet Implemented

The following major components from the original Node.js implementation need to be ported:

- **Full API Endpoints**: Complete REST API implementation (in progress)
- **Process Execution**: Plugin system and actual job runner
- **Multi-Server Clustering**: Server discovery, failover, and synchronization
- **User Authentication**: Full user management and bcrypt password hashing
- **WebSocket Support**: Real-time updates for the web UI
- **Email Notifications**: SMTP-based notification system
- **Web Hooks**: External notification webhooks
- **Web UI**: Frontend assets (currently unchanged from Node.js version)
- **Migration Tools**: Tools to migrate data from Node.js to Rust version

## Building and Running

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run
```

The server will start on port 3012 by default (configurable in `conf/config.json`).

### Test

```bash
cargo test
```

## Configuration

The Rust implementation uses the same `conf/config.json` format as the Node.js version. Key configuration options:

- `http_port`: HTTP server port (default: 3012)
- `base_app_url`: Base URL for the application
- `log_dir`: Directory for log files
- `queue_dir`: Directory for job queue data
- `secret_key`: Secret key for multi-server communication

## API Endpoints (Current)

- `GET /health` - Health check endpoint
- `GET /api/app/status` - Server status including active jobs count

## Dependencies

Key Rust dependencies:

- **tokio**: Async runtime
- **axum**: Web framework
- **serde/serde_json**: JSON serialization
- **chrono**: Date/time handling
- **tracing**: Logging
- **bcrypt**: Password hashing
- **reqwest**: HTTP client for webhooks

## Development Roadmap

1. ✅ Phase 1: Basic server infrastructure (COMPLETE)
2. ✅ Phase 2: Storage abstraction layer (COMPLETE)
3. ⏳ Phase 3: Scheduler implementation (IN PROGRESS)
4. ⏳ Phase 4: Job execution system
5. ⏳ Phase 5: API implementation
6. ⏳ Phase 6: Multi-server clustering
7. ⏳ Phase 7: User management
8. ⏳ Phase 8: WebSocket support
9. ⏳ Phase 9: Migration tools

## Differences from Node.js Version

- **Performance**: Expected to be faster due to Rust's compiled nature
- **Memory Safety**: Rust's ownership system prevents common memory bugs
- **Type Safety**: Strong static typing catches errors at compile time
- **Async Runtime**: Uses Tokio instead of Node.js event loop
- **Dependencies**: Native Rust crates instead of npm packages

## Contributing

This is an ongoing port. Contributions are welcome, especially for:

- Implementing missing components
- Writing tests
- Documentation
- Performance optimization

## License

MIT License - Same as the original Cronicle project
