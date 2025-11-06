# Migration Guide: Splitting the Monorepo

This guide provides a step-by-step approach for splitting the QiTech Control monorepo into multiple repositories, should that become necessary in the future.

## Prerequisites

Before considering a split:

1. **Framework Stability**: Ensure `ethercat-hal` and `control-core` APIs are stable
2. **Documentation**: All framework crates have comprehensive documentation
3. **Tests**: Framework crates have thorough test coverage
4. **Versioning Strategy**: Decide on semantic versioning approach
5. **Team Consensus**: All stakeholders agree on the split

## Recommended Approach: Gradual Migration

### Phase 1: Prepare Framework for External Use

**Goal**: Make framework crates publishable while keeping monorepo structure.

#### Step 1.1: Clean Up Framework Dependencies

Ensure framework crates only depend on:
- Each other (via path dependencies)
- Well-maintained external crates
- No `server` or machine-specific code

```bash
# Check dependency tree
cd control
cargo tree -p ethercat-hal
cargo tree -p control-core
```

#### Step 1.2: Add Package Metadata

Update `ethercat-hal/Cargo.toml` and `control-core/Cargo.toml`:

```toml
[package]
name = "ethercat-hal"
version = "0.1.0"
edition = "2024"
authors = ["QiTech Industries GmbH"]
license = "MIT OR Apache-2.0"  # Choose appropriate license
description = "Hardware abstraction layer for EtherCAT devices"
documentation = "https://docs.rs/ethercat-hal"
repository = "https://github.com/qitechgmbh/control-framework"
keywords = ["ethercat", "industrial", "automation", "beckhoff"]
categories = ["embedded", "hardware-support"]
readme = "README.md"
```

#### Step 1.3: Write Framework Documentation

For each framework crate:

1. Create `README.md` with:
   - Purpose and features
   - Installation instructions
   - Quick start example
   - Link to full documentation

2. Add crate-level documentation to `src/lib.rs`:
   ```rust
   //! # Ethercat HAL
   //!
   //! Hardware abstraction layer for EtherCAT devices.
   //!
   //! ## Quick Start
   //!
   //! ```no_run
   //! use ethercat_hal::{Device, Group};
   //! // ... example code
   //! ```
   ```

3. Document all public APIs with examples

#### Step 1.4: Test Framework Isolation

```bash
# Try building framework crates independently
cd ethercat-hal && cargo build
cd ../control-core && cargo build

# Run tests
cd ../ethercat-hal && cargo test
cd ../control-core && cargo test
```

#### Step 1.5: Publish to crates.io (Optional Dry Run)

```bash
cd ethercat-hal
cargo publish --dry-run

cd ../control-core
cargo publish --dry-run
```

### Phase 2: Extract Framework to Separate Repository

**Goal**: Move framework crates to a new repository.

#### Step 2.1: Create Framework Repository

```bash
# Create new repository
git clone https://github.com/qitechgmbh/control-framework.git
cd control-framework

# Set up structure
mkdir -p ethercat-hal ethercat-hal-derive control-core control-core-derive
touch Cargo.toml README.md LICENSE
```

#### Step 2.2: Copy Framework Code with History

Use `git filter-repo` to preserve commit history:

```bash
# Install git-filter-repo
pip install git-filter-repo

# Clone original repo
git clone https://github.com/qitechgmbh/control.git control-temp
cd control-temp

# Extract ethercat-hal with history
git filter-repo --subdirectory-filter ethercat-hal --force

# Push to framework repo
git remote add framework https://github.com/qitechgmbh/control-framework.git
git push framework main:ethercat-hal-history

# Repeat for other framework crates
```

#### Step 2.3: Set Up Framework Workspace

In `control-framework/Cargo.toml`:

```toml
[workspace]
members = [
    "ethercat-hal",
    "ethercat-hal-derive",
    "control-core",
    "control-core-derive",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
authors = ["QiTech Industries GmbH"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/qitechgmbh/control-framework"

[workspace.lints.clippy]
# ... copy from original
```

#### Step 2.4: Update Internal Dependencies

Update `control-core/Cargo.toml`:

```toml
[dependencies]
ethercat_hal = { version = "0.1.0", path = "../ethercat-hal" }
# ... other deps
```

#### Step 2.5: Add CI/CD

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo doc --no-deps

  publish:
    if: startsWith(github.ref, 'refs/tags/v')
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish --token ${CRATES_TOKEN}
        env:
          CRATES_TOKEN: ${{ secrets.CRATES_TOKEN }}
```

#### Step 2.6: Publish Framework to crates.io

```bash
cd control-framework

# Publish in dependency order
cd ethercat-hal-derive
cargo publish

cd ../ethercat-hal
cargo publish

cd ../control-core-derive
cargo publish

cd ../control-core
cargo publish
```

### Phase 3: Update QiTech Repository

**Goal**: Make QiTech repo depend on published framework crates.

#### Step 3.1: Remove Framework Code

In the original `control` repository:

```bash
cd control
git rm -r ethercat-hal ethercat-hal-derive control-core control-core-derive
```

#### Step 3.2: Update server/Cargo.toml

```toml
[dependencies]
# Replace path dependencies with crates.io dependencies
ethercat_hal = "0.1.0"
control_core = "0.1.0"
control_core_derive = "0.1.0"

# ... other deps remain same
```

#### Step 3.3: Update Workspace

Update root `Cargo.toml`:

```toml
[workspace]
members = [
    "server",
    "ethercat-eeprom-dump",
]
```

#### Step 3.4: Test Migration

```bash
cd server
cargo build
cargo test
```

#### Step 3.5: Update Documentation

Update `README.md`:

```markdown
# QiTech Control

This repository contains QiTech-specific machine implementations using the 
[control-framework](https://github.com/qitechgmbh/control-framework).

## Dependencies

- [ethercat-hal](https://crates.io/crates/ethercat-hal) - Hardware abstraction layer
- [control-core](https://crates.io/crates/control-core) - Control framework
```

### Phase 4: Extract UI (Optional)

**Goal**: Separate Electron UI into its own repository.

#### Step 4.1: Create UI Repository

```bash
git clone https://github.com/qitechgmbh/control-ui.git
cd control-ui

# Copy electron directory
cp -r ../control/electron/* .
```

#### Step 4.2: Make UI Generic

1. Remove QiTech-specific machine UIs or make them plugins
2. Create configuration for API endpoints
3. Add documentation for customization

#### Step 4.3: Publish as npm Package

```json
{
  "name": "@qitech/control-ui",
  "version": "1.0.0",
  "description": "Electron UI for industrial control systems",
  "main": "dist/main.js",
  "repository": "github:qitechgmbh/control-ui"
}
```

```bash
npm publish --access public
```

#### Step 4.4: Use UI in QiTech Repo

Option A: Git Submodule:
```bash
cd control
git rm -r electron
git submodule add https://github.com/qitechgmbh/control-ui.git electron
```

Option B: npm Package:
```json
{
  "dependencies": {
    "@qitech/control-ui": "^1.0.0"
  }
}
```

## Alternative: Gradual Separation without Multiple Repositories

If full split is not necessary, maintain logical separation within monorepo:

### Use Cargo Workspaces Effectively

```toml
[workspace]
members = [
    # Framework (don't depend on server)
    "framework/ethercat-hal",
    "framework/ethercat-hal-derive",
    "framework/control-core",
    "framework/control-core-derive",
    
    # Application (depends on framework)
    "app/server",
    
    # UI
    "ui/electron",
    
    # Utils
    "utils/ethercat-eeprom-dump",
]
```

### Enforce Boundaries with Linting

Add a CI check to ensure framework doesn't depend on application:

```bash
#!/bin/bash
# check-dependencies.sh

if grep -r "server" framework/*/Cargo.toml; then
  echo "Error: Framework should not depend on server!"
  exit 1
fi
```

## Rollback Plan

If migration causes issues:

### Roll Back Phase 3

```bash
cd control
git revert <migration-commit>
git push
```

### Revert to Path Dependencies

Update `server/Cargo.toml`:

```toml
[dependencies]
ethercat_hal = { path = "../ethercat-hal" }
control_core = { path = "../control-core" }
```

### Keep Framework Repo

The framework repository can remain as a mirror/backup even if not actively used.

## Post-Migration Maintenance

### Version Updates

When updating framework:

1. **Framework Repository**:
   ```bash
   cd control-framework
   # Make changes
   cargo test
   # Update version in Cargo.toml
   git tag v0.2.0
   git push --tags
   cargo publish
   ```

2. **QiTech Repository**:
   ```bash
   cd control/server
   # Update Cargo.toml
   ethercat_hal = "0.2.0"
   cargo update
   cargo test
   ```

### Breaking Changes

When framework has breaking changes:

1. Bump major version (0.1.0 → 0.2.0 or 1.0.0)
2. Write migration guide
3. Support old version for transition period
4. Update QiTech repo when ready

### Documentation

Keep both repositories' documentation in sync:
- Framework: API documentation
- QiTech: Machine implementation examples using framework

## Timeline Estimate

- **Phase 1** (Preparation): 2-4 weeks
  - Documentation and cleanup
  - Testing isolation
  
- **Phase 2** (Framework Extraction): 1-2 weeks
  - Repository setup
  - History migration
  - Publishing to crates.io
  
- **Phase 3** (QiTech Update): 1 week
  - Update dependencies
  - Testing
  - Documentation updates
  
- **Phase 4** (UI Extraction, Optional): 2-3 weeks
  - Repository setup
  - Generalization
  - Integration testing

**Total: 6-10 weeks** depending on scope

## Success Criteria

- ✅ Framework crates build independently
- ✅ All tests pass in both repositories
- ✅ Documentation is comprehensive
- ✅ CI/CD pipelines are working
- ✅ QiTech machines work with published framework
- ✅ Version management is clear
- ✅ Team is trained on new workflow

## Conclusion

Splitting the monorepo is a significant undertaking that should only be done when:
1. Framework is stable and well-documented
2. There's clear need for independent versioning
3. External users want to use the framework
4. Team has bandwidth to maintain multiple repositories

The gradual approach outlined here minimizes risk by:
- Preparing framework first (Phase 1 can be done immediately)
- Publishing framework while keeping monorepo (reversible)
- Only removing framework code once proven stable
- Maintaining clear rollback options

For now, continue with the monorepo approach while following best practices that keep the option open for future splitting.
