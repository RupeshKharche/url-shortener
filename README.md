# URL Shortener

A simple, fast URL shortener built with Rust, Axum, and SQLite.

## Features

- Create short URLs via REST API
- Redirect short codes to original URLs
- Health check endpoint
- SQLite database for persistence
- Configurable short code length
- **URL expiration (TTL)** — per-link or default TTL
- **Rate limiting** — per-IP limits on all endpoints

## Tech Stack

- **Language**: Rust (2024 edition)
- **Web Framework**: Axum
- **Database**: SQLite (via SQLx)
- **Async Runtime**: Tokio
- **Rate Limiting**: Governor (token bucket)

## Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- SQLite3 (usually pre-installed on most systems)

## Setup

1. **Clone the repository**
   ```bash
   git clone <your-repo-url>
   cd url-shortener
   ```

2. **Configure environment variables**
   Create a `.env` file in the project root:
   ```env
   DATABASE_URL=sqlite:urls.db
   BASE_URL=http://localhost:8080
   CODE_LENGTH=6
   DEFAULT_CODE_TTL_SECONDS=3600
   RATE_LIMIT_SHORTEN=10
   RATE_LIMIT_SHORTEN_WINDOW_SECS=60
   RATE_LIMIT_REDIRECT=100
   RATE_LIMIT_REDIRECT_WINDOW_SECS=60
   RATE_LIMIT_HEALTH=60
   RATE_LIMIT_HEALTH_WINDOW_SECS=60
   ```

   - `DATABASE_URL`: SQLite connection string (file path)
   - `BASE_URL`: Base URL for generated short links
   - `CODE_LENGTH`: Length of generated short codes (default: 6)
   - `DEFAULT_CODE_TTL_SECONDS`: Default link lifetime in seconds (default: 3600)
   - `RATE_LIMIT_*`: Per-endpoint rate limits (requests per window)

3. **Run database migrations**
   The database and tables are created automatically on first run via SQLx.

## Running the Project

```bash
# Development
cargo run

# Release (optimized)
cargo run --release
```

The server starts at `http://localhost:8080`.

## API Endpoints

| Method | Endpoint | Description | Rate Limit |
|--------|----------|-------------|------------|
| `GET` | `/health` | Health check | 60/min |
| `POST` | `/api/shorten` | Create a short URL | 10/min |
| `GET` | `/{code}` | Redirect to original URL | 100/min |

### Create Short URL

**Request:**
```bash
curl -X POST http://localhost:8080/api/shorten \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/very/long/url", "ttl_seconds": 7200}'
```

**Response:**
```json
{
  "short_url": "http://localhost:8080/aBc123",
  "expires_at": "2026-10-04T15:30:00Z"
}
```

- `ttl_seconds` (optional): Custom TTL in seconds. Uses `DEFAULT_CODE_TTL_SECONDS` if omitted.
- `expires_at`: RFC3339 timestamp when the link expires.

### Redirect

```bash
curl -L http://localhost:8080/aBc123
```
Redirects to the original URL with HTTP 302.

Returns **404 Not Found** if code doesn't exist or has expired (prevents enumeration).

### Rate Limit Headers

All responses include:
```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 9
```

On 429 Too Many Requests:
```
Retry-After: 45
```

## Load Testing

```bash
# Install k6
# Run load test
k6 run loadtest.js
```

## Project Structure

```
src/
├── main.rs          # Application entry point, routing, state
├── handlers.rs      # HTTP request handlers
├── models.rs        # Data models and DTOs
├── errors.rs        # Error types
├── config.rs        # Configuration from environment
└── rate_limit.rs    # Rate limiting middleware

migrations/
├── 001_create_urls.sql        # Database schema
└── 002_add_expires_at.sql     # Expiration column + index
```

## Database Schema

```sql
CREATE TABLE urls (
    id TEXT PRIMARY KEY NOT NULL,
    code TEXT NOT NULL UNIQUE,
    original_url TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TEXT NULL
);

CREATE INDEX idx_urls_code ON urls(code);
CREATE INDEX idx_urls_expires_at ON urls(expires_at);
```

## License

MIT