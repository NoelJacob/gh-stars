#!/bin/bash

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Build the application
build() {
  echo "Building gh-stars..."
  cd "$PROJECT_DIR"
  cargo build --release
  echo "Build complete! Binary at target/release/gh-stars"
}

# Run the application with specified arguments
run() {
  cd "$PROJECT_DIR"
  
  # Check if binary exists
  if [ ! -f "$PROJECT_DIR/target/release/gh-stars" ]; then
    echo "Binary not found. Building first..."
    build
  fi
  
  # Run the application with arguments passed to this script
  "$PROJECT_DIR/target/release/gh-stars" "$@"
}

# Show help information
show_help() {
  echo "GitHub Stars Manager - TUI application for managing starred repositories"
  echo ""
  echo "Usage:"
  echo "  ./script/build.sh [command] [options]"
  echo ""
  echo "Commands:"
  echo "  build                  Build the application"
  echo "  run [options]          Run the application"
  echo "  help                   Show this help message"
  echo ""
  echo "Application Options (for run command):"
  echo "  -u, --username         GitHub username"
  echo "  -t, --token            GitHub personal access token"
  echo "  --openai-key           OpenAI API key for AI suggestions"
  echo ""
  echo "Example:"
  echo "  ./script/build.sh run --username myuser --token ghp_123456 --openai-key sk-abc123"
}

# Make the script executable
chmod +x "$SCRIPT_DIR/build.sh"

# Main script logic
case "$1" in
  build)
    build
    ;;
  run)
    shift
    run "$@"
    ;;
  help|--help|-h)
    show_help
    ;;
  "")
    show_help
    ;;
  *)
    echo "Unknown command: $1"
    show_help
    exit 1
    ;;
esac
