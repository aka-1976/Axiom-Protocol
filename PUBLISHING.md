# 📦 AXIOM Protocol - Publishing Guide

Complete guide for publishing all AXIOM Protocol packages to public registries.

## 📋 Pre-Publishing Checklist

### Required Accounts & Credentials

- [ ] **Crates.io Account**
  - Register at: https://crates.io/
  - Get API token: https://crates.io/me
  - Login: `cargo login <token>`

- [ ] **PyPI Account**
  - Register at: https://pypi.org/account/register/
  - Create API token: https://pypi.org/manage/account/token/
  - Configure in `~/.pypirc`:
    ```ini
    [pypi]
    username = __token__
    password = <your-token>
    ```

- [ ] **npm Account**
  - Register at: https://www.npmjs.com/signup
  - Login: `npm login`
  - Enable 2FA (recommended)

- [ ] **Docker Hub Account** (Optional)
  - Register at: https://hub.docker.com/signup
  - Login: `docker login`

### Required Tools

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Python publishing tools
pip install --upgrade build twine

# Node.js and npm
# Visit: https://nodejs.org/

# Docker (optional)
# Visit: https://docs.docker.com/get-docker/
```

## 🚀 Quick Start - Automated Publishing

The easiest way to publish all packages:

```bash
# Make script executable
chmod +x publish-axiom.sh

# Run automated publishing
./publish-axiom.sh
```

This script will:
1. ✅ Run pre-publish checks
2. 🏗️ Build and test all packages
3. 📦 Publish to crates.io (with confirmation)
4. 🐍 Publish to PyPI (with confirmation)
5. 📦 Publish to npm (with confirmation)
6. 🐳 Build Docker images (optional)
7. 📊 Generate release artifacts

## 📦 Manual Publishing Steps

### 1. Rust Crate (crates.io)

```bash
# Verify Cargo.toml metadata
cat Cargo.toml | grep -A 10 "\[package\]"

# Build and test
cargo build --release
cargo test --release

# Dry-run (test without publishing)
cargo publish --dry-run

# Publish to crates.io
cargo publish
```

**First-time setup:**
```bash
# Get token from https://crates.io/me
cargo login <your-token>
```

### 2. Python Package (PyPI)

```bash
cd sdk/python

# Clean previous builds
rm -rf dist/ build/ *.egg-info/

# Build package
python3 -m build

# Check package
twine check dist/*

# Upload to TestPyPI (testing)
twine upload --repository testpypi dist/*

# Upload to PyPI (production)
twine upload dist/*

cd ../..
```

**First-time setup:**
Create `~/.pypirc`:
```ini
[distutils]
index-servers =
    pypi
    testpypi

[pypi]
username = __token__
password = pypi-...

[testpypi]
username = __token__
password = pypi-...
```

### 3. NPM Package

```bash
cd sdk/javascript

# Verify package.json
cat package.json

# Test package
npm pack --dry-run

# Login to npm
npm login

# Publish to npm
npm publish --access public

cd ../..
```

### 4. Docker Images

```bash
# Build images
docker build -t axiomprotocol/axiom-core:1.0.0 .
docker tag axiomprotocol/axiom-core:1.0.0 axiomprotocol/axiom-core:latest

# Test locally
docker run --rm axiomprotocol/axiom-core:1.0.0 --version

# Push to Docker Hub
docker login
docker push axiomprotocol/axiom-core:1.0.0
docker push axiomprotocol/axiom-core:latest
```

### 5. GitHub Release

```bash
# Create release artifacts
mkdir -p release-artifacts
cp target/release/axiom release-artifacts/axiom-linux-x64
cp target/release/axiom-wallet release-artifacts/axiom-wallet-linux-x64
cp target/release/axiom-supply release-artifacts/axiom-supply-linux-x64

# Generate checksums
cd release-artifacts
sha256sum * > SHA256SUMS.txt
cd ..

# Create GitHub Release
# 1. Go to: https://github.com/joker00099/Axiom-Protocol/releases/new
# 2. Tag: v1.0.0
# 3. Title: 🔺 AXIOM Protocol v1.0.0 - Production Release
# 4. Upload files from release-artifacts/
```

## 🔍 Verification

After publishing, verify packages are accessible:

### Crates.io
```bash
# Search for package
cargo search axiom-core

# Install from crates.io
cargo install axiom-core

# View on web
# https://crates.io/crates/axiom-core
```

### PyPI
```bash
# Search for package
pip search axiom-sdk

# Install from PyPI
pip install axiom-sdk

# View on web
# https://pypi.org/project/axiom-sdk/
```

### npm
```bash
# View package info
npm view axiom-sdk

# Install from npm
npm install axiom-sdk

# View on web
# https://www.npmjs.com/package/axiom-sdk
```

### Docker Hub
```bash
# Pull image
docker pull axiomprotocol/axiom-core:latest

# View on web
# https://hub.docker.com/r/axiomprotocol/axiom-core
```

## 📊 Package Statistics

After publishing, monitor your packages:

- **Crates.io**: https://crates.io/crates/axiom-core/stats
- **PyPI**: https://pypistats.org/packages/axiom-sdk
- **npm**: https://www.npmjs.com/package/axiom-sdk (built-in stats)
- **Docker Hub**: Docker Hub dashboard

## 🔄 Version Updates

When releasing a new version:

1. **Update version numbers:**
   ```bash
   # Cargo.toml
   version = "1.0.1"
   
   # sdk/python/setup.py
   version="1.0.1"
   
   # sdk/javascript/package.json
   "version": "1.0.1"
   ```

2. **Update CHANGELOG.md**

3. **Commit and tag:**
   ```bash
   git add -A
   git commit -m "🔺 Release v1.0.1"
   git tag -a v1.0.1 -m "Release v1.0.1"
   git push origin main --tags
   ```

4. **Re-run publishing:**
   ```bash
   ./publish-axiom.sh
   ```

## 🛡️ Security Best Practices

- ✅ Enable 2FA on all accounts
- ✅ Use API tokens instead of passwords
- ✅ Keep tokens in secure storage (not in code)
- ✅ Regularly rotate API tokens
- ✅ Review package contents before publishing
- ✅ Sign releases with GPG keys
- ✅ Monitor for security advisories

## 📝 Package Metadata Checklist

Ensure all packages have:

- [x] Clear description
- [x] MIT License
- [x] README with examples
- [x] Repository URL
- [x] Author information
- [x] Keywords/tags
- [x] Version number
- [x] Changelog
- [x] Dependencies listed
- [x] Build/installation instructions

## 🆘 Troubleshooting

### Cargo Publish Errors

**Error**: `the remote server responded with an error: You do not have permission to publish to this crate`
- Solution: The crate name might be taken. Try a different name or claim ownership.

**Error**: `file not found in crate`
- Solution: Add missing files to Cargo.toml `include` field or check `.gitignore`

### PyPI Upload Errors

**Error**: `403 Forbidden`
- Solution: Check your credentials in `~/.pypirc` and ensure 2FA is configured

**Error**: `File already exists`
- Solution: Cannot re-upload same version. Increment version number.

### npm Publish Errors

**Error**: `You must be logged in to publish packages`
- Solution: Run `npm login` first

**Error**: `Package name too similar to existing package`
- Solution: Choose a more unique name or add organization scope `@axiom/axiom-sdk`

### Docker Push Errors

**Error**: `denied: requested access to the resource is denied`
- Solution: Run `docker login` and ensure correct repository name

## 📚 Additional Resources

- **Cargo Book**: https://doc.rust-lang.org/cargo/reference/publishing.html
- **PyPI Guide**: https://packaging.python.org/tutorials/packaging-projects/
- **npm Publishing**: https://docs.npmjs.com/packages-and-modules/contributing-packages-to-the-registry
- **Docker Hub**: https://docs.docker.com/docker-hub/repos/

## 📞 Support

For publishing issues or questions:
- Open an issue: https://github.com/joker00099/Axiom-Protocol/issues
- Email: contact@axiom-protocol.org (if configured)

---

**Last Updated**: January 23, 2026  
**Version**: 1.0.0  
**Protocol**: 🏛️ AXIOM | 84M DECENTRALIZED
