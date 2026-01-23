#!/bin/bash
# 🔺 AXIOM Protocol - Complete Package Publishing Script
# Publishes all packages to their respective registries

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print banner
echo -e "${BLUE}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🏛️  AXIOM PROTOCOL - PACKAGE PUBLISHING SYSTEM"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${NC}"

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Function to print section headers
print_section() {
    echo -e "\n${BLUE}━━━ $1 ━━━${NC}\n"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Function to print warning
print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function to print error
print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to print info
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# ============================================================
# 1. PRE-PUBLISH CHECKS
# ============================================================
print_section "Pre-Publish Checks"

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    print_warning "Current branch is '$CURRENT_BRANCH', not 'main'"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_error "Publishing cancelled"
        exit 1
    fi
fi

# Check if working directory is clean
if ! git diff-index --quiet HEAD --; then
    print_warning "Working directory has uncommitted changes"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_error "Publishing cancelled"
        exit 1
    fi
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Rust"
    exit 1
fi
print_success "Rust/Cargo installed: $(cargo --version)"

# Check if Python is installed
if ! command -v python3 &> /dev/null; then
    print_error "Python3 not found"
    exit 1
fi
print_success "Python installed: $(python3 --version)"

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    print_error "Node.js not found"
    exit 1
fi
print_success "Node.js installed: $(node --version)"

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    print_error "npm not found"
    exit 1
fi
print_success "npm installed: $(npm --version)"

# ============================================================
# 2. BUILD AND TEST
# ============================================================
print_section "Build and Test"

print_info "Building Rust project..."
cargo build --release
print_success "Rust build successful"

print_info "Running tests..."
cargo test --release
print_success "All tests passed"

# ============================================================
# 3. RUST CRATE PUBLISHING (crates.io)
# ============================================================
print_section "Rust Crate Publishing (crates.io)"

print_info "Running cargo publish --dry-run..."
if cargo publish --dry-run; then
    print_success "Dry-run successful"
    
    echo
    read -p "Publish to crates.io? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Publishing to crates.io..."
        if cargo publish; then
            print_success "Successfully published axiom-core to crates.io!"
        else
            print_error "Failed to publish to crates.io"
            print_info "You may need to login first: cargo login <token>"
            print_info "Get your token from: https://crates.io/me"
        fi
    else
        print_warning "Skipped crates.io publishing"
    fi
else
    print_error "Dry-run failed - fix errors before publishing"
fi

# ============================================================
# 4. PYTHON PACKAGE PUBLISHING (PyPI)
# ============================================================
print_section "Python Package Publishing (PyPI)"

cd sdk/python

# Check if twine is installed
if ! command -v twine &> /dev/null; then
    print_info "Installing twine..."
    pip3 install --upgrade twine build
fi

print_info "Building Python package..."
python3 -m build

print_info "Checking package with twine..."
if twine check dist/*; then
    print_success "Package validation successful"
    
    echo
    read -p "Publish to PyPI? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Publishing to PyPI..."
        if twine upload dist/*; then
            print_success "Successfully published axiom-sdk to PyPI!"
        else
            print_error "Failed to publish to PyPI"
            print_info "You may need to configure credentials first"
            print_info "Create ~/.pypirc with your credentials"
        fi
    else
        print_warning "Skipped PyPI publishing"
    fi
else
    print_error "Package validation failed"
fi

cd "$SCRIPT_DIR"

# ============================================================
# 5. NPM PACKAGE PUBLISHING
# ============================================================
print_section "NPM Package Publishing"

cd sdk/javascript

print_info "Running npm pack (dry-run)..."
if npm pack --dry-run; then
    print_success "npm pack successful"
    
    echo
    read -p "Publish to npm registry? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Publishing to npm..."
        if npm publish --access public; then
            print_success "Successfully published axiom-sdk to npm!"
        else
            print_error "Failed to publish to npm"
            print_info "You may need to login first: npm login"
        fi
    else
        print_warning "Skipped npm publishing"
    fi
else
    print_error "npm pack failed"
fi

cd "$SCRIPT_DIR"

# ============================================================
# 6. CREATE GITHUB RELEASE
# ============================================================
print_section "GitHub Release"

print_info "Creating release artifacts..."
mkdir -p release-artifacts

# Copy binaries
if [ -f "target/release/axiom" ]; then
    cp target/release/axiom release-artifacts/axiom-linux-x64
    print_success "Copied axiom binary"
fi

if [ -f "target/release/axiom-wallet" ]; then
    cp target/release/axiom-wallet release-artifacts/axiom-wallet-linux-x64
    print_success "Copied axiom-wallet binary"
fi

if [ -f "target/release/axiom-supply" ]; then
    cp target/release/axiom-supply release-artifacts/axiom-supply-linux-x64
    print_success "Copied axiom-supply binary"
fi

# Create checksums
cd release-artifacts
sha256sum * > SHA256SUMS.txt
print_success "Generated checksums"
cd "$SCRIPT_DIR"

print_info "Release artifacts ready in: release-artifacts/"
print_info "Upload these to GitHub Release: https://github.com/joker00099/Axiom-Protocol/releases"

# ============================================================
# 7. DOCKER IMAGE (Optional)
# ============================================================
print_section "Docker Image Publishing (Optional)"

if [ -f "Dockerfile" ]; then
    echo
    read -p "Build and push Docker image? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Building Docker image..."
        docker build -t axiomprotocol/axiom-core:1.0.0 -t axiomprotocol/axiom-core:latest .
        
        print_info "Pushing to Docker Hub..."
        docker push axiomprotocol/axiom-core:1.0.0
        docker push axiomprotocol/axiom-core:latest
        print_success "Docker images pushed successfully"
    else
        print_warning "Skipped Docker image publishing"
    fi
fi

# ============================================================
# 8. SUMMARY
# ============================================================
print_section "Publishing Summary"

echo -e "${GREEN}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎉 AXIOM PROTOCOL PUBLISHING COMPLETE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${NC}"

echo -e "${BLUE}Package Links:${NC}"
echo "  📦 Rust Crate:   https://crates.io/crates/axiom-core"
echo "  🐍 PyPI:         https://pypi.org/project/axiom-sdk/"
echo "  📦 npm:          https://www.npmjs.com/package/axiom-sdk"
echo "  🐙 GitHub:       https://github.com/joker00099/Axiom-Protocol"
echo "  🐳 Docker Hub:   https://hub.docker.com/r/axiomprotocol/axiom-core"
echo

echo -e "${BLUE}Installation Commands:${NC}"
echo "  Rust:   cargo install axiom-core"
echo "  Python: pip install axiom-sdk"
echo "  Node:   npm install axiom-sdk"
echo "  Docker: docker pull axiomprotocol/axiom-core:latest"
echo

echo -e "${YELLOW}Next Steps:${NC}"
echo "  1. Create GitHub Release with artifacts from release-artifacts/"
echo "  2. Update documentation with installation instructions"
echo "  3. Announce on social media and community channels"
echo "  4. Monitor package download statistics"
echo "  5. Respond to issues and questions"
echo

print_success "All publishing tasks completed!"
echo
