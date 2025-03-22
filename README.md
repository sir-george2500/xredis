# xredis

xredis is a custom, lightweight implementation of a Redis-like server, designed to mimic the core functionality of Redis while keeping things simple and educational.

You might be wondering, "What is Redis, anyway?" Redis is a high-speed, in-memory database often used as a cache, a fast temporary data store, or even a message broker. At its core, Redis leverages the RESP (REdis Serialization Protocol) for data handling and operates on a single-threaded model with an I/O multiplexing approach for concurrency, making it exceptionally fast and efficient.

I was challenged to build this "lite" version of Redis, called `xredis`, as a way to help people deeply understand how Redis works under the hood—by building a simplified version from scratch.

## What xredis Is and What xredis Is Not

As stated earlier, `xredis` is a lightweight version of Redis, not a fully-featured implementation of the official Redis server. It captures the essence of Redis’s core concepts but omits some advanced features for simplicity. However, this foundation is robust enough to serve as a starting point for further development. If contributors wish to enhance this project, the possibilities are vast—sky’s the limit!

### What xredis Is:
- A learning tool to explore Redis’s internals.
- A minimal, functional in-memory key-value store.
- An implementation of the RESP protocol for client-server communication.
- A single-threaded server with async I/O using Rust’s Tokio runtime.

### What xredis Is Not:
- A production-ready replacement for Redis.
- A complete replica of all Redis features (e.g., clustering, Lua scripting, or replication).
- Optimized for performance at the scale of the official Redis server.

## Features of xredis

Here’s what `xredis` currently supports, mirroring the functionality of early Redis versions:

- **Basic Key-Value Operations**:
  - `SET key value [EX seconds, PX milli seconds , EAXT imestamp-seconds, PXAT timestamp-milliseconds ]`: Stores a string value with an optional expiration time.
  - `GET key`: Retrieves the value of a key (returns `(nil)` if not found or expired).
  - `EXISTS key [key ...]`: Checks if one or more keys exist (returns the count of existing, non-expired keys).
  - `DEL key [key ...]`: Deletes one or more keys (returns the count of deleted keys).

- **Integer Operations**:
  - `INCR key`: Increments the integer value of a key by 1 (initializes to 1 if not present).
  - `DECR key`: Decrements the integer value of a key by 1 (initializes to -1 if not present).

- **List Operations**:
  - `LPUSH key value [value ...]`: Inserts values at the head of a list.
  - `RPUSH key value [value ...]`: Inserts values at the tail of a list.

- **Persistence**:
  - `SAVE`: Saves the database state to disk (currently as a simple key-value file or JSON, depending on implementation).

- **RESP Protocol**: Implements the Redis Serialization Protocol for client compatibility (e.g., works with `redis-cli`).

- **Expiration**: Supports time-based key expiration, with lazy deletion on access (e.g., `GET` or `EXISTS` removes expired keys).

- **Concurrency**: Uses Rust’s async runtime (Tokio) for handling multiple client connections efficiently, despite being single-threaded.

## How It Works

`xredis` is built in Rust, leveraging its safety and performance features. The server:
1. Listens for connections on `127.0.0.1:6379` (Redis’s default port).
2. Parses incoming RESP commands using a custom parser.
3. Stores data in an in-memory `HashMap<String, ValueWithExpiry>`, where `ValueWithExpiry` can hold strings or lists with optional expiration timestamps.
4. Processes commands asynchronously using Tokio’s `TcpListener` and `Mutex` for thread-safe database access.
5. Persists data to disk on `SAVE` (currently a basic format, with potential for JSON serialization).

## Getting Started

### Prerequisites
- Rust (latest stable version recommended).
- `redis-cli` (optional, for testing compatibility).

### Running xredis
1. Clone the repository:
   ```bash
   git clone git@github.com:sir-george2500/xredis.git
   cd xredis
   ```
2. Run the testcases 
```bash 
   cargo test 
```
3. Run the project 
```bash
   cargo run 
```


## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request anytime.
