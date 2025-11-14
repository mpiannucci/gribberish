# Contributing

We welcome contributions to gribberish! This guide will help you get started.

## Development Setup

### Prerequisites

1. Install Rust (latest stable):
```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install Python 3.8+:
```bash
   # Using pyenv (recommended)
   pyenv install 3.11
   pyenv local 3.11
```

3. Install maturin:
```bash
   pip install maturin
```

### Clone and Setup
```bash
# Clone the repository
git clone https://github.com/mpiannucci/gribberish.git
cd gribberish

# Create a virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install development dependencies
pip install -e ".[dev]"
```

## Running Tests

### Rust Tests
```bash
# Run all Rust tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Python Tests
```bash
# Install test dependencies
pip install pytest pytest-cov

# Run Python tests
cd python
pytest

# With coverage
pytest --cov=gribberish
```

## Code Style

### Rust

We use rustfmt for formatting:
```bash
cargo fmt --all
cargo clippy --all-targets --all-features
```

### Python

We use black and ruff:
```bash
black python/
ruff python/
```

## Building Documentation
```bash
cd doc
make html
# Or on Windows: make.bat html
```

View at `doc/_build/html/index.html`

## Submitting Changes

1. Fork the repository
2. Create a feature branch (`git checkout -b feature-name`)
3. Make your changes
4. Run tests
5. Commit with descriptive message
6. Push to your fork
7. Open a Pull Request

## Release Process

Releases are automated via GitHub Actions when a tag is pushed:
```bash
git tag v0.3.0
git push origin v0.3.0
```

This triggers:
1. Rust crate publication to crates.io
2. Python wheel building for multiple platforms
3. PyPI publication
4. Documentation update