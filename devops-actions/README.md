# RustPress DevOps

Centralized GitHub Actions for the RustPress organization. This repository contains **composite actions** that can be used in any repository's workflows.

## Available Actions

| Action | Description |
|--------|-------------|
| `ci-rust` | Run Rust CI (check, test, clippy, fmt) |
| `ci-node` | Run Node.js CI (lint, test, build) |
| `build-rust` | Build Rust project for a specific target |
| `build-node` | Build Node.js project |
| `build-theme` | Package theme for release |
| `version-bump` | Determine version bump from commits |
| `create-release` | Create GitHub release with artifacts |
| `cleanup-releases` | Delete prereleases and drafts |
| `docker-build` | Build and push Docker image |

## Usage

### CI Workflow Example (Rust)

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustpress-net/rustpress-core-devops/actions/ci-rust@main
        with:
          run_tests: 'true'
          run_clippy: 'true'
          run_fmt: 'true'
```

### CI Workflow Example (Node.js)

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustpress-net/rustpress-core-devops/actions/ci-node@main
        with:
          run_tests: 'true'
          run_lint: 'true'
          run_build: 'true'
```

### Release Workflow Example (Rust)

```yaml
name: Release

on:
  push:
    branches: [main]
  workflow_dispatch:
    inputs:
      version_bump:
        description: 'Version bump type'
        required: true
        default: 'auto'
        type: choice
        options: [auto, patch, minor, major]
      release_type:
        description: 'Release type'
        required: true
        default: 'auto'
        type: choice
        options: [auto, release, pre-release, draft]

jobs:
  version:
    runs-on: ubuntu-latest
    outputs:
      new_version: ${{ steps.bump.outputs.new_version }}
      release_type: ${{ steps.bump.outputs.release_type }}
      should_release: ${{ steps.bump.outputs.should_release }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: rustpress-net/rustpress-core-devops/actions/version-bump@main
        id: bump
        with:
          version_bump: ${{ inputs.version_bump || 'auto' }}
          release_type: ${{ inputs.release_type || 'auto' }}
          version_file: 'Cargo.toml'

  build-linux:
    needs: version
    if: needs.version.outputs.should_release == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustpress-net/rustpress-core-devops/actions/build-rust@main
        with:
          target: x86_64-unknown-linux-gnu
          artifact_name: myapp-linux-x86_64
          include_admin_ui: 'true'

  build-windows:
    needs: version
    if: needs.version.outputs.should_release == 'true'
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustpress-net/rustpress-core-devops/actions/build-rust@main
        with:
          target: x86_64-pc-windows-msvc
          artifact_name: myapp-windows-x86_64

  build-macos:
    needs: version
    if: needs.version.outputs.should_release == 'true'
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustpress-net/rustpress-core-devops/actions/build-rust@main
        with:
          target: x86_64-apple-darwin
          artifact_name: myapp-macos-x86_64
      - uses: rustpress-net/rustpress-core-devops/actions/build-rust@main
        with:
          target: aarch64-apple-darwin
          artifact_name: myapp-macos-arm64

  release:
    needs: [version, build-linux, build-windows, build-macos]
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
      - uses: rustpress-net/rustpress-core-devops/actions/create-release@main
        with:
          version: ${{ needs.version.outputs.new_version }}
          release_type: ${{ needs.version.outputs.release_type }}
          artifact_pattern: 'myapp-*'
          release_name_prefix: 'MyApp '

  cleanup:
    needs: [version, release]
    if: needs.version.outputs.release_type == 'release'
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: rustpress-net/rustpress-core-devops/actions/cleanup-releases@main
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}

  docker:
    needs: [version, release]
    if: needs.version.outputs.release_type == 'release'
    runs-on: ubuntu-latest
    permissions:
      packages: write
    steps:
      - uses: actions/checkout@v4
      - uses: rustpress-net/rustpress-core-devops/actions/docker-build@main
        with:
          version: ${{ needs.version.outputs.new_version }}
          image_name: myapp
          github_token: ${{ secrets.GITHUB_TOKEN }}
```

### Release Workflow Example (Theme)

```yaml
name: Release

on:
  push:
    branches: [main]
  workflow_dispatch:
    inputs:
      version_bump:
        description: 'Version bump type'
        default: 'auto'
        type: choice
        options: [auto, patch, minor, major]
      release_type:
        description: 'Release type'
        default: 'auto'
        type: choice
        options: [auto, release, pre-release, draft]

jobs:
  version:
    runs-on: ubuntu-latest
    outputs:
      new_version: ${{ steps.bump.outputs.new_version }}
      release_type: ${{ steps.bump.outputs.release_type }}
      should_release: ${{ steps.bump.outputs.should_release }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: rustpress-net/rustpress-core-devops/actions/version-bump@main
        id: bump
        with:
          version_file: 'theme.json'

  build:
    needs: version
    if: needs.version.outputs.should_release == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustpress-net/rustpress-core-devops/actions/build-theme@main
        with:
          version: ${{ needs.version.outputs.new_version }}
          artifact_name: my-theme

  release:
    needs: [version, build]
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
      - uses: rustpress-net/rustpress-core-devops/actions/create-release@main
        with:
          version: ${{ needs.version.outputs.new_version }}
          release_type: ${{ needs.version.outputs.release_type }}
```

## Release Type Control

Control release type via commit messages:

- `[release]` - Creates an official release, triggers cleanup of prereleases/drafts
- `[pre-release]` - Creates a prerelease
- `[draft]` - Creates a draft release
- No tag - No release is created

## Version Bumping

Version is automatically determined from conventional commits:

| Pattern | Bump Type |
|---------|-----------|
| `feat!:` or `BREAKING CHANGE` | Major |
| `feat:` or `feature:` | Minor |
| Everything else | Patch |

## License

MIT License - RustPress Team
