#!/bin/bash

set -euo pipefail

echo "Building Eclipta..."

# Step 1: Build Frontend (Vite)
echo "Building frontend..."
cd frontend
npm install
npm run build
cd ..

# Step 2: Build Backend
echo "Building backend..."
cargo build --release --package backend

# Step 3: Package
DIST_DIR="dist"
echo "Packaging into $DIST_DIR/"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

cp target/release/backend "$DIST_DIR/eclipta-backend"
cp -r frontend/dist/* "$DIST_DIR/"

# Step 4: Optional CLI build
if [ -f "eclipta-cli/Cargo.toml" ]; then
  echo "🛠 Building CLI..."
  cargo build --release --package eclipta-cli
  cp target/release/eclipta-cli "$DIST_DIR/eclipta-cli"
else
  echo "CLI not found, skipping."
fi

echo ""
echo "Build complete!"
echo "Run backend with: ./dist/eclipta-backend"
