#!/bin/bash

# Setup script for Linux Command Library

set -e

echo "🚀 Linux Command Library Setup"
echo "=============================="

# Check if database exists
if [ ! -f "database.db" ]; then
    echo "📥 Downloading database..."
    wget https://github.com/SimonSchubert/LinuxCommandLibrary/raw/master/assets/database.db -O database.db
    echo "✅ Database downloaded successfully"
else
    echo "✅ Database already exists"
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "🔨 Building project..."
cargo build --release

echo ""
echo "✅ Setup complete!"
echo ""
echo "To run the application:"
echo "  ./target/release/LinuxCommandLibrary"
echo ""
echo "Or use Docker:"
echo "  docker-compose up -d"
echo ""
echo "Access the application at http://localhost:8080"
