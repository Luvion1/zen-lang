# GitHub Actions Workflows

This directory contains comprehensive CI/CD workflows for the Zen programming language project.

## Workflows Overview

### 1. **ci.yml** - Main CI Pipeline
- **Triggers**: Push to main/develop, PRs, weekly schedule
- **Jobs**:
  - Code quality checks (formatting, clippy, documentation)
  - Cross-platform testing (Ubuntu, Windows, macOS)
  - Security scanning with Trivy and CodeQL
  - Performance benchmarking
  - Cross-platform builds (x86_64, ARM64)
  - Documentation generation and deployment

### 2. **release.yml** - Release Management
- **Triggers**: Git tags (v*)
- **Features**:
  - Automated release creation
  - Cross-platform binary builds
  - Asset uploads with checksums
  - Changelog generation
  - Pre-release detection

### 3. **security.yml** - Security Auditing
- **Triggers**: Weekly schedule, pushes, PRs
- **Features**:
  - Dependency vulnerability scanning
  - Security audit with cargo-audit
  - CodeQL static analysis
  - Dependency review for PRs

### 4. **benchmarks.yml** - Performance Testing
- **Triggers**: Push to main, PRs, weekly schedule
- **Features**:
  - Compilation speed benchmarks
  - Execution performance testing
  - Binary size analysis
  - Automated PR comments with results

### 5. **docs.yml** - Documentation
- **Triggers**: Push to main, PRs
- **Features**:
  - API documentation generation
  - GitHub Pages deployment
  - Link checking
  - Beautiful documentation site

### 6. **examples-test.yml** - Example Validation
- **Triggers**: Push, PRs, daily schedule
- **Features**:
  - Cross-platform example testing
  - Compilation validation
  - Documentation consistency checks
  - Test report generation

## Workflow Features

### 🔄 **Continuous Integration**
- ✅ Multi-platform testing (Linux, macOS, Windows)
- ✅ Multiple Rust versions (stable, beta)
- ✅ Comprehensive test suite
- ✅ Code quality enforcement
- ✅ Security scanning

### 🚀 **Continuous Deployment**
- ✅ Automated releases on tags
- ✅ Cross-platform binary distribution
- ✅ Documentation deployment
- ✅ Package registry publishing

### 📊 **Performance Monitoring**
- ✅ Compilation speed tracking
- ✅ Execution benchmarks
- ✅ Binary size monitoring
- ✅ Performance regression detection

### 🔒 **Security**
- ✅ Dependency vulnerability scanning
- ✅ Static code analysis
- ✅ Security audit automation
- ✅ SARIF report generation

### 📚 **Documentation**
- ✅ Automated API docs
- ✅ GitHub Pages deployment
- ✅ Example validation
- ✅ Link checking

## Secrets Required

For full functionality, configure these GitHub secrets:

```
GITHUB_TOKEN          # Automatically provided
CARGO_REGISTRY_TOKEN  # For crates.io publishing (optional)
```

## Caching Strategy

All workflows use intelligent caching:
- Cargo registry and git dependencies
- Build artifacts
- LLVM installations
- Documentation builds

## Artifact Management

Workflows generate and store:
- Cross-platform binaries
- Test reports
- Benchmark results
- Documentation sites
- Security scan results

## Monitoring and Notifications

- ✅ Build status badges
- ✅ Performance regression alerts
- ✅ Security vulnerability notifications
- ✅ Automated PR comments
- ✅ Release notifications

## Customization

### Adding New Platforms
Edit the matrix in `ci.yml` and `release.yml`:
```yaml
matrix:
  include:
    - os: ubuntu-latest
      target: aarch64-unknown-linux-gnu
      name: zen-linux-arm64
```

### Adding New Benchmarks
Extend `benchmarks.yml` with additional hyperfine commands:
```yaml
- name: Custom benchmark
  run: hyperfine './target/release/zen compile custom_test.zen'
```

### Modifying Security Scans
Update `security.yml` to add new security tools or modify scan parameters.

## Best Practices

1. **Branch Protection**: Enable required status checks
2. **Dependency Updates**: Use Dependabot for automated updates
3. **Security**: Regular security audits and vulnerability scanning
4. **Performance**: Monitor benchmarks for regressions
5. **Documentation**: Keep docs in sync with code changes

## Troubleshooting

### Common Issues
- **LLVM Installation**: Ensure LLVM is properly installed on all platforms
- **Timeout Issues**: Adjust timeout values for long-running examples
- **Cache Issues**: Clear cache if builds become inconsistent
- **Permission Issues**: Ensure proper GitHub token permissions

### Debug Steps
1. Check workflow logs in GitHub Actions tab
2. Verify LLVM installation steps
3. Test examples locally on target platform
4. Check artifact uploads and downloads
5. Validate secret configuration
