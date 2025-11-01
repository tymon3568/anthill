# GitHub Actions Workflows

This directory contains GitHub Actions workflows for CI/CD automation.

## Workflows Overview

| Workflow | File | Purpose | Triggers |
|----------|------|---------|----------|
| **CI Testing Pipeline** | `ci-testing.yml` | Comprehensive testing (unit, integration, security, coverage) | Push, PR |
| **Test Coverage** | `test-coverage.yml` | Coverage reporting with Codecov | Push, PR |
| **OpenAPI Export** | `openapi-export.yml` | API documentation generation | Push, PR |

---

## CI Testing Pipeline

**File**: `ci-testing.yml` ⭐ **PRIMARY WORKFLOW**

### Overview

Comprehensive automated testing pipeline that runs on every push and pull request.

### Pipeline Stages

```
Lint & Format
    ├─→ Unit Tests ───┐
    ├─→ Integration Tests ─┤
    ├─→ Security Tests ────┤─→ Coverage Report ─→ Test Summary
    └─→ Build Check ───────┘
```

### Jobs

1. **Lint & Format Check** (~2 min) - Code quality validation
2. **Unit Tests** (~5 min) - Fast tests with mocks
3. **Integration Tests** (~8 min) - Real PostgreSQL database
4. **Security Tests** (~6 min) - SQL injection, tenant isolation, auth
5. **Coverage Report** (~10 min) - LLVM coverage + Codecov
6. **Build Check** (~15 min) - All services compile
7. **Test Summary** - Aggregate results + PR comments

### Usage

**Local Simulation**:
```bash
# Run complete pipeline
./scripts/ci-helper.sh all

# Individual stages
./scripts/ci-helper.sh lint
./scripts/ci-helper.sh unit --verbose
./scripts/ci-helper.sh integration
./scripts/ci-helper.sh coverage --open
```

### Documentation

See [docs/testing/CI_CD_PIPELINE.md](../../docs/testing/CI_CD_PIPELINE.md) for detailed documentation.

---

## Test Coverage Workflow

**File**: `test-coverage.yml`

Legacy coverage workflow with service-specific breakdown. Runs in parallel with CI testing pipeline.

---

## OpenAPI Export Workflow

**File**: `openapi-export.yml`

### Purpose

Automatically exports and merges OpenAPI specifications from all microservices whenever code is pushed to `services/` directory.

### Workflow Steps

1. **Export Individual Specs**: Each service builds with `--features export-spec` to generate its OpenAPI YAML
2. **Merge Specs**: All individual specs are merged into a single `api.yaml` using Redocly CLI
3. **Commit Back**: Merged spec is committed back to the repository (on master branch only)
4. **PR Comments**: On pull requests, comments with spec file sizes

### Triggers

- **Push** to `master`, `main`, or `develop` branches when `services/**` changes
- **Pull Request** when `services/**` changes

### Artifacts

- Individual service specs (retention: 1 day)
- Merged `api.yaml` (retention: 30 days)

### Usage

The workflow runs automatically. To test locally:

```bash
# Export single service
cd services/user_service
cargo build --features export-spec

# Export all services
for service in services/*/; do
  (cd "$service" && cargo build --features export-spec)
done
```

### Outputs

```
shared/openapi/
├── user.yaml          # User service spec
├── inventory.yaml     # Inventory service spec
├── order.yaml         # Order service spec
├── payment.yaml       # Payment service spec
├── integration.yaml   # Integration service spec
└── api.yaml           # 🔀 Merged final spec (for frontend)
```

### Configuration

To add a new service to the workflow:

1. Create a new job in `openapi-export.yml`:
```yaml
export-myservice:
  name: Export MyService Spec
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
      with:
        workspaces: "services/myservice"
    - run: |
        cd services/myservice
        cargo build --release --features export-spec
    - uses: actions/upload-artifact@v4
      with:
        name: myservice-spec
        path: shared/openapi/myservice.yaml
        retention-days: 1
```

2. Add to `merge-specs` needs:
```yaml
merge-specs:
  needs:
    - export-user-service
    - export-myservice  # Add here
```

3. Update merge command to include new spec:
```bash
redocly bundle shared/openapi/user.yaml \
  shared/openapi/myservice.yaml \
  ...
```

### Troubleshooting

**Merge fails?**
- Check Redocly CLI output in workflow logs
- Verify all specs have unique `operationId` values
- Ensure specs are valid OpenAPI 3.1.0 format

**No specs generated?**
- Verify service has `export-spec` feature in Cargo.toml
- Check that export logic exists in service's main.rs
- Review build logs for compilation errors

**Commit not pushing?**
- Ensure workflow has write permissions (Settings → Actions → Workflow permissions)
- Check `[skip ci]` is in commit message to prevent infinite loops
- Verify branch protection rules allow bot commits
