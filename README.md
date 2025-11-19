# Linux Command Library (Web API)

A modern, high-performance web API server for Linux command reference, built with Rust and inspired by the original Kotlin project. This implementation provides a complete REST API with enhanced features and a beautiful, responsive web interface.

## 🚀 Features

### Core Functionality
- **7680+ Linux Commands**: Complete database of Linux manual pages and commands
- **Advanced Search**: Full-text search with auto-completion and suggestions
- **Categorized Browsing**: 23+ categories from System to Text Editors
- **Random Tips**: Daily Linux tips and tricks
- **TLDR Support**: Quick command summaries for busy users

### API Features
- **RESTful Design**: Clean, intuitive API endpoints
- **Comprehensive Error Handling**: Detailed error responses with proper HTTP status codes
- **Request Validation**: Input validation and sanitization
- **Rate Limiting Ready**: Infrastructure for rate limiting (easily add middleware)
- **CORS Support**: Configurable Cross-Origin Resource Sharing

### Technical Improvements
- **High Performance**: Built with Rust and Actix-web for maximum speed
- **Proper Error Handling**: Custom error types with detailed error messages
- **Comprehensive Logging**: Structured logging with different levels (debug, info, warn, error)
- **Database Schema Validation**: Automatic validation of database structure on startup
- **Environment Configuration**: Flexible configuration via environment variables
- **Modern Frontend**: Responsive, accessible web interface with dark mode support

## 📋 API Endpoints

### Core Endpoints
```
GET  /health                    # Health check
GET  /api/stats                 # Application statistics
GET  /api/categories            # List all categories
GET  /api/categories/detailed   # Categories with descriptions and icons
```

### Search Endpoints
```
GET  /api/search?q=query        # Search commands
GET  /api/suggestions?q=query   # Auto-completion suggestions
GET  /api/popular               # Popular commands
```

### Command Endpoints
```
GET  /api/commands/{id}         # Get command details
GET  /api/category/{name}       # Commands by category
GET  /api/random-tip           # Get random Linux tip
```

### Frontend
```
GET  /                          # Serve web interface
```

## 🛠️ Installation & Setup

### Prerequisites
- Rust 1.70+ (for local development)
- Docker & Docker Compose (for containerized deployment - **recommended**)

### Quick Start with Docker (Recommended)

The easiest way to run the application is using Docker. The database is automatically downloaded during the build process.

```bash
# Clone the repository
git clone <repository-url>
cd LinuxCommandLibrary

# Build and run with docker-compose
docker-compose up -d

# Or build and run manually
docker build -t linux-command-library .
docker run -p 8080:8080 linux-command-library
```

Access the application at `http://localhost:8080`

### Local Development Setup

```bash
# Clone the repository
git clone <repository-url>
cd LinuxCommandLibrary

# Download the database (if not present)
wget https://github.com/SimonSchubert/LinuxCommandLibrary/raw/master/assets/database.db

# Build the project
cargo build --release

# Run with default settings
./target/release/LinuxCommandLibrary

# Or with custom configuration
DATABASE_PATH=custom.db SERVER_ADDR=0.0.0.0:3000 ./target/release/LinuxCommandLibrary
```

### Environment Variables
- `DATABASE_PATH`: Path to SQLite database (default: `database.db`)
- `SERVER_ADDR`: Server bind address (default: `0.0.0.0:8080`)
- `ENABLE_CORS`: Enable CORS support (default: `true`)
- `RUST_LOG`: Log level (default: `info`)

## 🐳 Docker Deployment

### Using Docker Compose (Recommended)
```bash
docker-compose up -d
```

### Manual Docker Commands
```bash
# Build the image
docker build -t linux-command-library .

# Run the container
docker run -d \
  --name linux-command-library \
  -p 8080:8080 \
  -e RUST_LOG=info \
  linux-command-library

# View logs
docker logs -f linux-command-library

# Stop the container
docker stop linux-command-library
```

### Docker Features
- **Multi-stage build**: Optimized image size
- **Automatic database download**: No manual database setup required
- **Health checks**: Built-in health monitoring
- **Alpine-based**: Minimal image footprint (~50MB)
- **Production-ready**: Includes proper signal handling and graceful shutdown

## 💻 Frontend Interface

The web interface has been completely redesigned with modern features and enhanced user experience:

### Modern Design Features
- **Responsive Layout**: Works perfectly on desktop, tablet, and mobile
- **Dark Mode Support**: Automatic dark/light theme switching with smooth transitions
- **Smooth Animations**: Subtle transitions and hover effects
- **Accessibility**: Proper ARIA labels and keyboard navigation
- **SEO Optimized**: Meta tags and structured data
- **Print-Friendly**: Optimized print styles for documentation

### Enhanced Search Experience
- **Smart Search**: Real-time search with debouncing
- **Search History**: Stores last 10 searches with quick access
- **Clear Button**: One-click search field clearing
- **Keyboard Shortcuts**: Press `/` to quickly focus search
- **Search Indicators**: Visual feedback during search

### Navigation Enhancements
- **URL Routing**: Hash-based routing for shareable URLs
- **Breadcrumb Navigation**: Clear navigation path in category views
- **Back to Top**: Floating button appears when scrolling down
- **A-Z Navigation**: Quick alphabetical jump for command list

### Command Details Modal
- **Copy Actions**: Copy command name, share link, copy sections
- **Section Management**: Individual copy buttons for each section
- **Share Functionality**: Native share API support with fallback
- **Keyboard Navigation**: Full keyboard support with Esc to close
- **Focus Management**: Proper focus restoration when closing

### User Experience Features
- **Toast Notifications**: Modern toast messages for actions
- **Loading States**: Skeleton loaders and spinners
- **Empty States**: Helpful messages when no results found
- **Category Icons**: Visual category representation with Lucide icons
- **Smooth Scrolling**: Enhanced scroll behavior throughout

### Performance Optimizations
- **Debounced Search**: Prevents excessive API calls (200ms delay)
- **Lazy Icon Loading**: Icons loaded only when needed
- **LocalStorage**: Efficient caching of search history and theme
- **Optimized Icons**: Using Lucide icon library for consistency
- **Progressive Enhancement**: Core functionality works without JavaScript

## 🔧 Technical Architecture

### Backend Architecture
- **Framework**: Actix-web (async, high-performance)
- **Database**: SQLite with connection pooling
- **Error Handling**: Custom error types with proper HTTP responses
- **Logging**: Structured logging with `env_logger`
- **Configuration**: Environment-based configuration

### Data Models
```rust
// Enhanced command model with TLDR support
struct CommandDetail {
    id: i64,
    name: String,
    category: String,  // Translated from numeric category
    description: String,
    sections: Vec<CommandSection>,
    tldr: Option<String>,  // Quick summary
}

// Category information with descriptions and icons
struct BasicCategory {
    id: i64,
    title: String,
    position: i64,
    description: Option<String>,
    icon: Option<String>,
}
```

### Error Handling
The application implements comprehensive error handling:
- **Database Errors**: Proper SQLite error handling
- **Validation Errors**: Input validation with clear messages
- **Not Found Errors**: Graceful handling of missing resources
- **Internal Errors**: Server errors with appropriate logging

## 📊 Performance

### Benchmarks
- **Response Time**: Sub-100ms response times for most endpoints
- **Memory Usage**: Efficient memory management with Rust
- **Concurrent Requests**: Handles thousands of concurrent connections
- **Database Queries**: Optimized queries with proper indexing

### Optimizations
- **Connection Pooling**: Efficient database connection management
- **Query Optimization**: Indexed database queries
- **Response Caching**: Infrastructure for response caching
- **Async Processing**: Non-blocking I/O operations

## 🧪 Testing

Run the test suite:
```bash
cargo test
```

Run with coverage:
```bash
cargo tarpaulin --out Html
```

## 🔍 Database Schema

The application uses the following database schema (based on the original Kotlin project):

### Tables
- `Command`: Main command information
- `CommandSection`: Detailed command sections (SYNOPSIS, DESCRIPTION, etc.)
- `Tip`: Linux tips and tricks
- `TipSection`: Tip content sections
- `BasicCategory`: Command categories for browsing
- `BasicGroup`: Command groups within categories
- `BasicCommand`: Basic command examples

### Sample Data
```sql
-- Commands table
CREATE TABLE Command (
    id INTEGER PRIMARY KEY,
    category INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL
);

-- Command sections (TLDR, SYNOPSIS, DESCRIPTION, OPTIONS, etc.)
CREATE TABLE CommandSection (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    command_id INTEGER NOT NULL
);
```

## 🔧 Development

### Project Structure
```
LinuxCommandLibrary/
├── src/
│   ├── main.rs           # Main application file
│   └── index.html        # Web interface
├── Cargo.toml            # Rust dependencies
├── database.db           # SQLite database
└── README.md
```

### Adding New Features
1. **API Endpoints**: Add new route handlers in `main.rs`
2. **Data Models**: Define new structs with proper serialization
3. **Database Queries**: Add prepared statements with error handling
4. **Frontend**: Update `index.html` with new JavaScript functions

### Code Style
- Follow Rust naming conventions
- Use meaningful variable and function names
- Add comments for complex logic
- Handle all error cases explicitly

## 📚 API Documentation

### Response Format
All API responses follow a consistent format:
```json
{
    "success": true,
    "data": { ... },
    "message": null
}
```

### Error Responses
```json
{
    "success": false,
    "data": null,
    "message": "Error description"
}
```

### Example Requests

**Search Commands:**
```bash
curl "http://localhost:8080/api/search?q=grep"
```

**Get Command Details:**
```bash
curl "http://localhost:8080/api/commands/123"
```

**Get Categories:**
```bash
curl "http://localhost:8080/api/categories/detailed"
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## 📄 License

This project is licensed under the Apache License 2.0 - see the original Kotlin project for details.

## 🙏 Acknowledgments

- Original Kotlin project by Simon Schubert
- Linux manual page contributors
- Rust and Actix-web communities

## 📞 Support

For issues and questions:
1. Check the existing issues
2. Create a new issue with detailed information
3. Include steps to reproduce any bugs

---

Thanks for [origin](https://github.com/SimonSchubert/LinuxCommandLibrary)

**Built with ❤️ using Rust and inspired by the original Kotlin project**