# Axum Boilerplate

A production-ready Rust API boilerplate built with Axum, featuring JWT authentication, SQLite database, Redis integration, and comprehensive testing.

## Table of Contents

- [Features](#features)
- [Technology Stack](#technology-stack)
- [Architecture Overview](#architecture-overview)
- [Quick Start](#quick-start)
- [Installation & Setup](#installation--setup)
- [Environment Configuration](#environment-configuration)
- [API Endpoints](#api-endpoints)
- [Authentication Flow](#authentication-flow)
- [Database Schema](#database-schema)
- [Testing](#testing)
- [Project Structure](#project-structure)
- [Development Guidelines](#development-guidelines)
- [Future Plans](#future-plans)
- [Contributing](#contributing)
- [License](#license)

## Features

- **JWT Authentication** - Secure authentication with access and refresh tokens
- **Cookie-based Token Management** - HTTP-only cookies for enhanced security
- **Database Integration** - SQLite with SQLx and automatic migrations
- **Redis Support** - Token blacklisting and allowlisting with Redis
- **Comprehensive Testing** - Full test suite for all authentication flows
- **Structured Logging** - Detailed logging with tracing
- **Password Hashing** - Secure password storage with bcrypt
- **CORS Support** - Configurable CORS for web applications
- **Security Middleware** - Authentication middleware for protected routes

## Technology Stack

- **Framework**: [Axum](https://github.com/tokio-rs/axum) - Modern async web framework
- **Database**: [SQLite](https://sqlite.org/) with [SQLx](https://github.com/launchbadge/sqlx) - Type-safe SQL
- **Caching**: [Redis](https://redis.io/) - Token management and caching
- **Authentication**: [JWT](https://jwt.io/) with [jsonwebtoken](https://crates.io/crates/jsonwebtoken)
- **Password Hashing**: [bcrypt](https://crates.io/crates/bcrypt)
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **Serialization**: [Serde](https://serde.rs/)
- **Logging**: [Tracing](https://github.com/tokio-rs/tracing)

## Architecture Overview

The boilerplate follows a clean, modular architecture:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HTTP Client   │    │      Redis      │    │     SQLite      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Axum Router   │────│  JWT Service    │────│   Auth Service  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Auth Middleware│    │ Cookie Service  │    │   User Model    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

- **API Layer**: HTTP endpoints and request/response handling
- **Middleware**: Authentication and request processing
- **Services**: Business logic for authentication, JWT management, and cookies
- **Models**: Data structures and database operations
- **Database**: SQLite for persistent storage, Redis for token management

## Quick Start

1. **Clone the repository**
```bash
git clone <your-repo-url>
cd axum-boilerplate
```

2. **Set up environment**
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. **Start dependencies**
```bash
# Start Redis (using Docker)
docker run -d -p 6379:6379 redis:alpine

# Or install Redis locally and start it
redis-server
```

4. **Run the application**
```bash
cargo run
```

5. **Test the API**
```bash
# Register a new user
curl -X POST http://localhost:3000/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"password123"}'

# Login
curl -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
```

## Installation & Setup

### Prerequisites

- **Rust** (1.75+ recommended)
- **Redis** server
- **SQLite** (included with SQLx)

### Step-by-step Setup

1. **Install Rust** (if not already installed):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

2. **Clone and navigate to the project**:
```bash
git clone <your-repo-url>
cd axum-boilerplate
```

3. **Install Redis**:

   **On macOS** (using Homebrew):
   ```bash
   brew install redis
   brew services start redis
   ```

   **On Ubuntu/Debian**:
   ```bash
   sudo apt-get install redis-server
   sudo systemctl start redis-server
   ```

   **Using Docker**:
   ```bash
   docker run -d --name redis -p 6379:6379 redis:alpine
   ```

4. **Set up environment variables**:
```bash
cp .env.example .env
```

5. **Run database migrations**:
```bash
cargo run
# Migrations run automatically on startup
```

6. **Start the server**:
```bash
cargo run
```

The server will start on `http://localhost:3000`.

## Environment Configuration

Create a `.env` file in the root directory with the following variables:

```env
DATABASE_URL=sqlite:db.sqlite
SECRET_KEY=mySuperSecretKey
REDIS_URL=redis://localhost:6379
```

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | SQLite database connection string | `sqlite:db.sqlite` | Yes |
| `SECRET_KEY` | JWT signing secret (use a strong random string) | - | Yes |
| `REDIS_URL` | Redis connection URL | `redis://localhost:6379` | Yes |

**Security Note**: Use a strong, randomly generated `SECRET_KEY` in production. You can generate one using:
```bash
openssl rand -base64 32
```

## API Endpoints

### Public Endpoints

#### POST `/register`
Register a new user account.

**Request Body:**
```json
{
  "username": "string",
  "email": "string",
  "password": "string"
}
```

**Response (200 OK):**
```json
{
  "message": "Registration successful",
  "success": true,
  "id": 1
}
```

#### POST `/login`
Authenticate user and receive tokens via HTTP-only cookies.

**Request Body:**
```json
{
  "email": "string",
  "password": "string"
}
```

**Response (200 OK):**
```json
{
  "message": "Login successful",
  "success": true
}
```

**Response Headers:**
- `Set-Cookie: access_token=...` (HTTP-only, 15 min expiry)
- `Set-Cookie: refresh_token=...` (HTTP-only, 7 days expiry)

#### POST `/refresh`
Refresh access token using refresh token from cookies.

**Response (200 OK):**
```json
{
  "message": "Tokens refreshed successfully",
  "success": true
}
```

#### POST `/logout`
Revoke refresh token and clear authentication cookies.

**Response (200 OK):**
```json
{
  "message": "Logout successful",
  "success": true
}
```

### Protected Endpoints

All protected endpoints require valid authentication cookies.

#### GET `/me`
Get current authenticated user information.

**Response (200 OK):**
```json
{
  "id": 1,
  "username": "testuser",
  "email": "test@example.com"
}
```

### Error Responses

All endpoints may return the following error status codes:

- `400 Bad Request` - Invalid request data
- `401 Unauthorized` - Invalid or expired authentication
- `500 Internal Server Error` - Server error

## Authentication Flow

The boilerplate implements a secure JWT-based authentication system:

### 1. Registration & Login
1. User registers with username, email, and password
2. Password is hashed using bcrypt before storage
3. On login, credentials are verified
4. JWT token pair is generated (access + refresh)
5. Tokens are set as HTTP-only cookies

### 2. Token Management
- **Access Token**: Short-lived (15 minutes), used for API requests
- **Refresh Token**: Long-lived (7 days), used to generate new access tokens
- **Token Storage**: HTTP-only cookies for enhanced security
- **Token Blacklisting**: Redis-based blacklist for revoked tokens

### 3. Request Authentication
1. Client includes cookies in requests automatically
2. Auth middleware extracts access token from cookies
3. Token is verified and validated
4. User information is attached to request context
5. Protected endpoints receive authenticated user data

### 4. Token Refresh
1. When access token expires, client calls `/refresh`
2. Refresh token is validated against Redis allowlist
3. New token pair is generated and old refresh token is revoked
4. New tokens are set as cookies

## Database Schema

### Users Table

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Fields:**
- `id`: Unique user identifier
- `email`: User's email address (unique)
- `username`: User's chosen username (unique)
- `password_hash`: bcrypt hashed password
- `created_at`: Account creation timestamp
- `updated_at`: Last modification timestamp

## Testing

The project includes comprehensive tests covering all authentication flows.

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test module
cargo test tests::auth

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Test Structure

The test suite is located in `src/tests/` and includes:

#### `tests/auth.rs`
- **User Registration**: Valid registration with unique email and username
- **User Login**: Successful authentication with valid credentials
- **Token Refresh**: Access token renewal using refresh token
- **User Logout**: Token revocation and cookie clearing
- **Invalid Credentials**: Proper error handling for wrong passwords
- **Protected Routes**: Authentication middleware validation
- **Current User**: Retrieving authenticated user information

#### `tests/helpers.rs`
Test utilities and setup functions:
- Database setup and teardown
- Test application creation
- HTTP request helpers
- Cookie extraction utilities

### Test Features

- **Database Isolation**: Each test uses a fresh in-memory database
- **Redis Integration**: Tests include Redis token management
- **HTTP Testing**: Full HTTP request/response cycle testing
- **Cookie Handling**: Proper cookie-based authentication testing
- **Error Scenarios**: Comprehensive error condition testing

### Running Individual Tests

```bash
# Test user registration
cargo test test_register_success

# Test login flow
cargo test test_login_success

# Test token refresh
cargo test test_refresh_token_success

# Test protected routes
cargo test test_get_current_user
```

## Project Structure

```
axum-boilerplate/
├── src/
│   ├── api/                    # HTTP endpoints
│   │   ├── auth.rs            # Authentication endpoints
│   │   ├── user.rs            # User management endpoints
│   │   └── mod.rs
│   ├── middleware/             # HTTP middleware
│   │   ├── auth.rs            # Authentication middleware
│   │   └── mod.rs
│   ├── models/                 # Data models
│   │   ├── user.rs            # User model and database operations
│   │   ├── jwt.rs             # JWT token structures
│   │   └── mod.rs
│   ├── services/               # Business logic
│   │   ├── auth_service.rs    # Authentication service
│   │   ├── jwt_service.rs     # JWT token management
│   │   ├── cookie_service.rs  # Cookie utilities
│   │   └── mod.rs
│   ├── db/                     # Database configuration
│   │   ├── mod.rs             # Database connection setup
│   │   └── redis.rs           # Redis store implementation
│   ├── tests/                  # Test modules
│   │   ├── auth.rs            # Authentication tests
│   │   ├── helpers.rs         # Test utilities
│   │   └── mod.rs
│   └── main.rs                 # Application entry point
├── migrations/                 # Database migrations
│   └── 20240417000000_create_users_table.sql
├── docs/                       # Documentation (future)
├── Cargo.toml                  # Dependencies and project config
├── .env.example               # Environment variables template
├── .gitignore                 # Git ignore rules
└── README.md                  # This file
```

### Module Responsibilities

- **`api/`**: HTTP request handling and response formatting
- **`middleware/`**: Request processing and authentication
- **`models/`**: Data structures and database operations
- **`services/`**: Business logic and external service integration
- **`db/`**: Database connection and Redis store management
- **`tests/`**: Comprehensive test suite

## Development Guidelines

### Adding New Endpoints

1. **Create the handler** in the appropriate `api/` module
2. **Add routing** in `main.rs` `create_router` function
3. **Add authentication** if needed using the auth middleware
4. **Write tests** in the corresponding test module

### Code Style

- Use `cargo fmt` for consistent formatting
- Use `cargo clippy` for linting
- Follow Rust naming conventions
- Add comprehensive error handling
- Include logging for debugging

### Database Changes

1. Create new migration files in `migrations/`
2. Use descriptive filenames with timestamps
3. Test migrations with fresh databases
4. Update models to reflect schema changes

## Future Plans

This boilerplate is actively being developed. Planned features include:

- **Password Reset** - Email-based password recovery
- **Email Verification** - Account activation via email
- **Role-Based Access Control** - User roles and permissions
- **Email Service Integration** - SMTP configuration and templates
- **Configuration Management** - More configurable settings
- **Docker Support** - Containerization setup
- **Rate Limiting** - API rate limiting middleware
- **API Documentation** - OpenAPI/Swagger integration
- **Code Cleanup** - Refactoring and optimization

## Contributing

Contributions are welcome! This project is designed to be a solid foundation for Rust API development, and community input helps make it better.

### How to Contribute

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Make your changes** with tests
4. **Run the test suite** (`cargo test`)
5. **Commit your changes** (`git commit -m 'Add amazing feature'`)
6. **Push to the branch** (`git push origin feature/amazing-feature`)
7. **Open a Pull Request**

### Contribution Guidelines

- Write tests for new features
- Follow existing code style and patterns
- Update documentation for new features
- Ensure all tests pass before submitting
- Add appropriate logging and error handling

### Areas for Contribution

- Additional authentication methods
- Performance improvements
- Security enhancements
- Documentation improvements
- Test coverage expansion
- Example applications

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

The MIT License is a permissive license that allows you to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the software, provided that the above copyright notice and this permission notice appear in all copies.

---

**Built with ❤️ and Rust**

For questions, issues, or suggestions, please open an issue on GitHub.
