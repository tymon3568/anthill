# Codecov Setup Guide

## 🎯 Overview

This project uses [Codecov](https://codecov.io) for code coverage tracking with **cargo-llvm-cov** for Rust code coverage generation.

## 📊 Current Status

[![codecov](https://codecov.io/gh/tymon3568/anthill/branch/master/graph/badge.svg)](https://codecov.io/gh/tymon3568/anthill)

## 🚀 Setup Instructions

### Step 1: Sign up for Codecov

1. Go to https://codecov.io/
2. Click "Sign up with GitHub"
3. Authorize Codecov to access your repositories

### Step 2: Add Repository

1. Go to https://app.codecov.io/gh/tymon3568
2. Click "Add new repository"
3. Find and select **anthill**

### Step 3: Get Upload Token

1. In Codecov, go to your repository settings
2. Navigate to "Settings" → "General"
3. Copy the "Upload Token" (looks like: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`)

### Step 4: Add Token to GitHub Secrets

1. Go to https://github.com/tymon3568/anthill/settings/secrets/actions
2. Click "New repository secret"
3. Name: `CODECOV_TOKEN`
4. Value: Paste the upload token from Step 3
5. Click "Add secret"

### Step 5: Enable GitHub Actions

1. Go to https://github.com/tymon3568/anthill/actions
2. If workflows are disabled, click "I understand my workflows, go ahead and enable them"

### Step 6: Verify Setup

Push a commit to trigger the workflow:

```bash
cd /home/arch/anthill
git add .github/workflows/test-coverage.yml codecov.yml .github/CODECOV_SETUP.md
git commit -m "ci: add codecov integration for test coverage"
git push origin master
```

Check workflow status:
- https://github.com/tymon3568/anthill/actions

View coverage report:
- https://codecov.io/gh/tymon3568/anthill

## 📈 Features

### Parallel Coverage Generation

The workflow runs coverage for:
- ✅ **Workspace** (all services combined)
- ✅ **User Service** (individual)
- ✅ **Inventory Service** (individual)

### Coverage Targets

- **Project Coverage**: 80% (minimum)
- **Patch Coverage**: 75% (minimum)
- **Precision**: 2 decimal places

### Service Flags

Each service has its own flag for granular tracking:
- `unittests` - All workspace tests
- `user-service` - User service only
- `inventory-service` - Inventory service only
- `order-service` - Order service only
- `payment-service` - Payment service only
- `integration-service` - Integration service only

## 🔧 Local Testing

### Install cargo-llvm-cov

```bash
cargo install cargo-llvm-cov
```

### Generate Coverage Report

```bash
# Full workspace
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info

# Specific service
cd services/user-service
cargo llvm-cov --all-features --lcov --output-path ../../lcov-user.info

# HTML report (for local viewing)
cargo llvm-cov --workspace --all-features --html
open target/llvm-cov/html/index.html
```

### View Coverage Locally

```bash
# Generate HTML report
cargo llvm-cov --workspace --all-features --open
```

## 📝 Configuration

### codecov.yml

The `codecov.yml` file in the repository root configures:

- **Coverage targets**: 80% project, 75% patch
- **Ignore patterns**: Test files, examples, benchmarks
- **Flags**: Per-service coverage tracking
- **Comments**: Automatic PR comments with coverage diff

### GitHub Actions Workflow

`.github/workflows/test-coverage.yml` runs on:

- **Push** to master/main/develop branches
- **Pull requests** with Rust code changes
- **Paths**: `services/**/*.rs`, `shared/**/*.rs`, `**/Cargo.toml`

## 🎨 Codecov Badges

### Add to README.md

```markdown
## Code Coverage

[![codecov](https://codecov.io/gh/tymon3568/anthill/branch/master/graph/badge.svg)](https://codecov.io/gh/tymon3568/anthill)
```

### Coverage Graph

```markdown
[![codecov](https://codecov.io/gh/tymon3568/anthill/branch/master/graphs/sunburst.svg)](https://codecov.io/gh/tymon3568/anthill)
```

## 🐛 Troubleshooting

### "Missing upload token"

- Ensure `CODECOV_TOKEN` is set in GitHub Secrets
- Check spelling: it must be exactly `CODECOV_TOKEN`

### "No coverage files found"

- Ensure tests exist in the service
- Check that `cargo test` runs successfully
- Verify `lcov.info` is being generated

### "Coverage decreased"

- This is expected for new features without tests
- Add tests to cover new code
- Coverage threshold is 2% - small decreases are allowed

### Workflow not running

- Check `.github/workflows/test-coverage.yml` exists
- Verify GitHub Actions are enabled for the repository
- Ensure changed files match the `paths` filter

## 📚 Resources

- **Codecov Docs**: https://docs.codecov.com/
- **cargo-llvm-cov**: https://github.com/taiki-e/cargo-llvm-cov
- **Rust Testing**: https://doc.rust-lang.org/book/ch11-00-testing.html
- **GitHub Actions**: https://docs.github.com/en/actions

## 🎉 Success Criteria

- ✅ Codecov token configured in GitHub Secrets
- ✅ Workflow runs on push/PR
- ✅ Coverage reports uploaded to Codecov
- ✅ PR comments show coverage diff
- ✅ Codecov badge shows in README
- ✅ Coverage visible at https://codecov.io/gh/tymon3568/anthill
