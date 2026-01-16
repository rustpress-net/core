# RustPress DevOps

Centralized GitHub Actions and DevOps workflows for the RustPress organization. This repository contains **reusable workflows** that are called from other repositories in the organization.

> **Note**: This repository is an **abstract container** for workflows. No actions are executed on this repository directly - all workflows are designed to be called from other repositories.

## Available Reusable Workflows

### Release Workflows

| Workflow | File | Description | Use For |
|----------|------|-------------|---------|
| Rust Release | `release-rust.yml` | Multi-platform Rust builds with optional Docker | Core, plugins |
| Node.js Release | `release-node.yml` | Node.js/npm project releases | Admin UI |
| Theme Release | `release-theme.yml` | Theme package releases | All themes |
| Documentation Release | `release-docs.yml` | Documentation releases | Docs, crates |

### CI Workflows

| Workflow | File | Description | Use For |
|----------|------|-------------|---------|
| Rust CI | `ci-rust.yml` | Rust checks, tests, clippy, format | All Rust projects |
| Node.js CI | `ci-node.yml` | Node.js lint, test, build | All Node.js projects |

## How to Use Reusable Workflows

### Basic Concept

GitHub reusable workflows allow you to define a workflow once and call it from multiple repositories. The calling repository (caller) uses `uses:` to reference the reusable workflow (callee) in this devops repository.

### Syntax

```yaml
jobs:
  job-name:
    uses: rustpress-net/rustpress-core-devops/.github/workflows/<workflow-file>@main
    with:
      input1: value1
      input2: value2
    secrets: inherit
```

### Key Points

1. **Reference Format**: `<org>/<repo>/.github/workflows/<file>@<ref>`
2. **Branch Reference**: Use `@main` for the latest stable workflows
3. **Secrets**: Use `secrets: inherit` to pass all secrets from caller to callee
4. **Inputs**: Pass configuration via `with:` block

---

## Release Workflow Usage

### For Rust Projects (Core, Plugins)

Create `.github/workflows/release.yml` in your repository:

```yaml
name: Release

on:
  push:
    branches: [main, master]
  workflow_dispatch:
    inputs:
      version_bump:
        description: 'Version bump type'
        required: true
        default: 'auto'
        type: choice
        options:
          - auto
          - patch
          - minor
          - major
      release_type:
        description: 'Release type'
        required: true
        default: 'auto'
        type: choice
        options:
          - auto
          - release
          - pre-release
          - draft

jobs:
  release:
    uses: rustpress-net/rustpress-core-devops/.github/workflows/release-rust.yml@main
    with:
      version_bump: ${{ inputs.version_bump || 'auto' }}
      release_type: ${{ inputs.release_type || 'auto' }}
      build_docker: true           # Set to false for plugins
      include_admin_ui: true       # Set to false for plugins
    secrets: inherit
```

#### Rust Release Inputs

| Input | Type | Default | Description |
|-------|------|---------|-------------|
| `version_bump` | string | `auto` | Version bump: `auto`, `patch`, `minor`, `major` |
| `release_type` | string | `auto` | Release type: `auto`, `release`, `pre-release`, `draft` |
| `build_docker` | boolean | `true` | Build and push Docker image |
| `docker_image_name` | string | repo name | Custom Docker image name |
| `include_admin_ui` | boolean | `false` | Include Admin UI in build |
| `admin_ui_ref` | string | from file | Admin UI version override |

### For Node.js Projects (Admin UI)

```yaml
name: Release

on:
  push:
    branches: [main, master]
  workflow_dispatch:
    inputs:
      version_bump:
        description: 'Version bump type'
        required: true
        default: 'auto'
        type: choice
        options:
          - auto
          - patch
          - minor
          - major
      release_type:
        description: 'Release type'
        required: true
        default: 'auto'
        type: choice
        options:
          - auto
          - release
          - pre-release
          - draft

jobs:
  release:
    uses: rustpress-net/rustpress-core-devops/.github/workflows/release-node.yml@main
    with:
      version_bump: ${{ inputs.version_bump || 'auto' }}
      release_type: ${{ inputs.release_type || 'auto' }}
    secrets: inherit
```

#### Node.js Release Inputs

| Input | Type | Default | Description |
|-------|------|---------|-------------|
| `version_bump` | string | `auto` | Version bump type |
| `release_type` | string | `auto` | Release type |
| `node_version` | string | `20` | Node.js version |
| `build_command` | string | `npm run build` | Build command |
| `dist_folder` | string | `dist` | Distribution folder to package |

### For Theme Projects

```yaml
name: Release

on:
  push:
    branches: [main, master]
  workflow_dispatch:
    inputs:
      version_bump:
        description: 'Version bump type'
        required: true
        default: 'auto'
        type: choice
        options:
          - auto
          - patch
          - minor
          - major
      release_type:
        description: 'Release type'
        required: true
        default: 'auto'
        type: choice
        options:
          - auto
          - release
          - pre-release
          - draft

jobs:
  release:
    uses: rustpress-net/rustpress-core-devops/.github/workflows/release-theme.yml@main
    with:
      version_bump: ${{ inputs.version_bump || 'auto' }}
      release_type: ${{ inputs.release_type || 'auto' }}
    secrets: inherit
```

#### Theme Release Inputs

| Input | Type | Default | Description |
|-------|------|---------|-------------|
| `version_bump` | string | `auto` | Version bump type |
| `release_type` | string | `auto` | Release type |

### For Documentation Projects

```yaml
name: Release

on:
  push:
    branches: [main, master]
  workflow_dispatch:
    inputs:
      version_bump:
        description: 'Version bump type'
        required: true
        default: 'auto'
        type: choice
        options:
          - auto
          - patch
          - minor
          - major
      release_type:
        description: 'Release type'
        required: true
        default: 'auto'
        type: choice
        options:
          - auto
          - release
          - pre-release
          - draft

jobs:
  release:
    uses: rustpress-net/rustpress-core-devops/.github/workflows/release-docs.yml@main
    with:
      version_bump: ${{ inputs.version_bump || 'auto' }}
      release_type: ${{ inputs.release_type || 'auto' }}
    secrets: inherit
```

#### Documentation Release Inputs

| Input | Type | Default | Description |
|-------|------|---------|-------------|
| `version_bump` | string | `auto` | Version bump type |
| `release_type` | string | `auto` | Release type |
| `build_command` | string | empty | Build command (if docs need building) |
| `dist_folder` | string | `docs` | Folder to package |

---

## CI Workflow Usage

### For Rust Projects

Create `.github/workflows/ci.yml` in your repository:

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  ci:
    uses: rustpress-net/rustpress-core-devops/.github/workflows/ci-rust.yml@main
    with:
      run_tests: true
      run_clippy: true
      run_fmt: true
      postgres_required: true    # Set based on your needs
      redis_required: true       # Set based on your needs
    secrets: inherit
```

#### Rust CI Inputs

| Input | Type | Default | Description |
|-------|------|---------|-------------|
| `run_tests` | boolean | `true` | Run cargo tests |
| `run_clippy` | boolean | `true` | Run clippy linting |
| `run_fmt` | boolean | `true` | Check code formatting |
| `postgres_required` | boolean | `false` | Start PostgreSQL service |
| `redis_required` | boolean | `false` | Start Redis service |

### For Node.js Projects

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  ci:
    uses: rustpress-net/rustpress-core-devops/.github/workflows/ci-node.yml@main
    with:
      run_tests: true
      run_lint: true
      run_build: true
    secrets: inherit
```

#### Node.js CI Inputs

| Input | Type | Default | Description |
|-------|------|---------|-------------|
| `node_version` | string | `20` | Node.js version |
| `run_tests` | boolean | `true` | Run tests |
| `run_lint` | boolean | `true` | Run linting |
| `run_build` | boolean | `true` | Run build |
| `test_command` | string | `npm test` | Custom test command |
| `lint_command` | string | `npm run lint` | Custom lint command |
| `build_command` | string | `npm run build` | Custom build command |

---

## Release Type Control

### Commit Message Tags

Control release type via commit messages:

- `[release]` - Creates an official release, cleans up all prereleases/drafts
- `[pre-release]` - Creates a prerelease
- `[draft]` - Creates a draft release
- No tag - No release is created

### Examples

```bash
# Create an official release
git commit -m "feat: Add new feature [release]"

# Create a pre-release
git commit -m "feat: Add experimental feature [pre-release]"

# Create a draft for review
git commit -m "feat: WIP feature [draft]"

# Normal commit (no release)
git commit -m "fix: Bug fix"
```

### What Happens on Release

When `[release]` is used:
1. Creates official release with binaries
2. Builds and pushes Docker image (if enabled)
3. **Deletes ALL prereleases and drafts** to keep releases clean

---

## Version Bumping

Version is automatically determined from commit messages using conventional commits:

| Pattern | Bump Type |
|---------|-----------|
| `feat!:` or `BREAKING CHANGE` | Major |
| `feat:` or `feature:` | Minor |
| Everything else | Patch |

You can also manually specify version bump via workflow dispatch.

---

## Repository Setup Checklist

When adding a new repository to use centralized workflows:

1. **Identify project type**: Rust, Node.js, Theme, or Docs
2. **Create `.github/workflows/release.yml`** with appropriate caller workflow
3. **Create `.github/workflows/ci.yml`** with appropriate CI workflow
4. **Ensure version file exists**:
   - Rust: `Cargo.toml` with `version = "x.y.z"`
   - Node.js: `package.json` with `"version": "x.y.z"`
   - Theme: `theme.json` with `"version": "x.y.z"`
   - Docs: `VERSION` file or `package.json`
5. **Test with a draft release**: Use `[draft]` commit to verify setup

---

## Troubleshooting

### Workflow Not Triggering

- Ensure the branch name matches the trigger (`main` or `master`)
- Check that commit message includes release type tag
- Verify `secrets: inherit` is included in caller workflow

### Permission Errors

- The caller repository must have Actions enabled
- `secrets: inherit` passes `GITHUB_TOKEN` automatically
- For Docker push, ensure packages permission is configured

### Version Not Updating

- Check that version file exists and is in correct format
- Verify conventional commit patterns in commit messages

---

## License

MIT License - RustPress Team
