# URL Shortener

A simple, fast URL shortener built with Rust, Axum, and SQLite.

## Features

- Create short URLs via REST API
- Redirect short codes to original URLs
- Health check endpoint
- SQLite database for persistence
- Configurable short code length

## Tech Stack

- **Language**: Rust (2024 edition)
- **Web Framework**: Axum
- **Database**: SQLite (via SQLx)
- **Async Runtime**: Tokio

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
   ```

   - `DATABASE_URL`: SQLite connection string (file path)
   - `BASE_URL`: Base URL for generated short links
   - `CODE_LENGTH`: Length of generated short codes (default: 6)

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

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `POST` | `/api/shorten` | Create a short URL |
| `GET` | `/{code}` | Redirect to original URL |

### Create Short URL

**Request:**
```bash
curl -X POST http://localhost:8080/api/shorten \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/very/long/url"}'
```

**Response:**
```json
{
  "short_url": "http://localhost:8080/aBc123",
  "code": "aBc123"
}
```

### Redirect

```bash
curl -L http://localhost:8080/aBc123
```
Redirects to the original URL with HTTP 302.

## Project Structure

```
src/
├── main.rs       # Application entry point, routing, state
├── handlers.rs   # HTTP request handlers
├── models.rs     # Data models and DTOs
└── errors.rs     # Error types

migrations/
└── 001_create_urls.sql  # Database schema
```

## Database Schema

```sql
CREATE TABLE urls (
    id TEXT PRIMARY KEY NOT NULL,
    code TEXT NOT NULL UNIQUE,
    original_url TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_urls_code ON urls(code);
```