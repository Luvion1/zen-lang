# CI/CD Workflows

This directory contains GitHub Actions workflows for the Zen programming language project.

## Workflows

### 🔄 ci.yml - Continuous Integration
- **Triggers**: Push/PR to main/develop
- **Jobs**:
  - **test**: Run tests on Ubuntu, macOS, Windows (Rust stable only)
  - **fmt**: Check code formatting with rustfmt
  - **clippy**: Lint code with clippy (no warnings allowed)
  - **build**: Build release binaries and test them
  - **install-script**: Test install.sh syntax and help

### 🚀 build.yml - Release Builds
- **Triggers**: Git tags (v*), manual dispatch
- **Jobs**:
  - **build**: Cross-platform release builds (Linux x64, macOS x64/ARM64, Windows x64)
  - **create-release**: Create GitHub release with binaries and installation instructions

### 🧪 examples-test.yml - Example Testing
- **Triggers**: Push/PR to main/develop
- **Jobs**:
  - **test-examples**: Compile all examples, run safe ones with timeout
  - **install-test**: Test install script on Ubuntu/macOS

### 🔒 security.yml - Security Checks
- **Triggers**: Push to main, PRs, weekly schedule
- **Jobs**:
  - **audit**: Run cargo audit for security vulnerabilities
  - **dependency-check**: Check dependency tree
  - **license-check**: Verify license compliance

### 📚 docs.yml - Documentation
- **Triggers**: Push to main, manual dispatch
- **Jobs**:
  - **build-docs**: Generate Rust docs + GitHub Pages site
  - **deploy-docs**: Deploy to GitHub Pages

## Key Features

✅ **No Caching**: All workflows use `cargo clean` for fresh builds
✅ **Cross-Platform**: Test on Linux, macOS, Windows
✅ **Security**: Regular security audits and dependency checks
✅ **Documentation**: Auto-generated docs with GitHub Pages
✅ **Release Automation**: Automatic binary builds and GitHub releases
✅ **Example Testing**: Ensure all examples compile and run safely

## Usage

Workflows run automatically on push/PR. For releases:

```bash
git tag v0.0.2
git push origin v0.0.2
```

This will trigger the build workflow and create a GitHub release with binaries.
