# Release Process

This document outlines the release process for the `@spruceid/opencred-dc-api` npm package.

## Overview

The package uses automated CI/CD workflows for testing, validation, and publishing. The release process is triggered by pushing git tags with a specific format.

## Prerequisites

Before releasing, ensure you have:

1. **npm Registry Access**: The `NPM_SPRUCEID_PUBLISH_TOKEN` secret must be configured in the GitHub repository
2. **Version Updated**: The `package.json` version must match the git tag version
3. **Tests Passing**: All CI checks must be green on the main branch

## Release Workflow

### 1. Prepare the Release

Update the version in `package.json`:

```bash
cd npm-package
npm version patch  # or minor/major
```

This will:
- Update `package.json` version
- Create a git commit with the version change
- Create a local git tag (but not push it yet)

### 2. Push the Version Commit

```bash
git push origin main
```

Wait for CI to pass before proceeding.

### 3. Trigger the Release

Push the tag created by `npm version` with the correct format:

```bash
# If npm version created tag "v1.2.3", rename it to the expected format
git tag npm/v1.2.3 v1.2.3
git tag -d v1.2.3  # Delete the old tag
git push origin npm/v1.2.3
```

Or create the tag manually:

```bash
# For version 1.2.3
git tag npm/v1.2.3
git push origin npm/v1.2.3
```

### 4. Monitor the Release

The tag push will trigger the `npm_cd` workflow which will:

1. **Validate** that the package.json version matches the tag
2. **Build** the WASM binary and JavaScript bindings
3. **Publish** to npm registry with public access

Monitor the workflow at: `https://github.com/spruceid/dc-api/actions`

## Tag Format

Tags must follow the format: `npm/v[MAJOR].[MINOR].[PATCH]`

Examples:
- `npm/v0.1.0` - Initial release
- `npm/v0.1.1` - Patch release
- `npm/v0.2.0` - Minor release  
- `npm/v1.0.0` - Major release

## Automated Workflows

### CI Workflow (`npm_ci`)

Runs on every push/PR to validate:
- ✅ WASM builds successfully
- ✅ Package structure is correct
- ✅ No source files in build output
- ✅ Package can be loaded
- ✅ Rust tests pass
- ✅ Security audits pass
- ✅ Package size is reasonable

### CD Workflow (`npm_cd`)

Runs on tag push to:
1. Validate version consistency
2. Set up Node.js and Rust environment
3. Install wasm-bindgen-cli
4. Build the complete package
5. Publish to npm

## Manual Release (Fallback)

If the automated workflow fails, you can publish manually:

```bash
cd npm-package

# Clean and build
npm run clean
npm run build

# Verify the build
npm pack --dry-run

# Login to npm (if needed)
npm login

# Publish
npm publish --access public
```

## Post-Release

After a successful release:

1. **Verify Publication**: Check that the package appears on [npmjs.com](https://www.npmjs.com/package/@spruceid/opencred-dc-api)
2. **Test Installation**: Try installing the package in a test project
3. **Update Documentation**: Update README or other docs if needed
4. **Create GitHub Release**: Optionally create a GitHub release with changelog

## Troubleshooting

### Common Issues

**Version Mismatch Error:**
```
Error: Package version in package.json doesn't match tag
```
Solution: Ensure the version in `package.json` exactly matches the tag (without the `npm/v` prefix).

**WASM Build Failure:**
```
Error: WASM file not found
```
Solution: Check that Rust toolchain and wasm-bindgen are properly installed in CI.

**npm Authentication Error:**
```
Error: Unable to authenticate
```
Solution: Verify the `NPM_SPRUCEID_PUBLISH_TOKEN` secret is correctly configured.

**Package Already Published:**
```
Error: Cannot publish over existing version
```
Solution: You cannot republish the same version. Increment the version and try again.

### Debug Commands

Check current tags:
```bash
git tag -l "npm/v*"
```

Verify package contents before release:
```bash
npm pack --dry-run
```

Test the built package locally:
```bash
node -e "console.log(require('./dist/dc_api_wasm.js'))"
```

## Security Notes

- The npm token has publish access only
- All builds happen in isolated GitHub Actions runners
- The WASM binary is built from source during CI, not pre-compiled
- Dependencies are audited automatically in CI

## Version Strategy

- **Patch** (x.x.X): Bug fixes, security updates
- **Minor** (x.X.x): New features, API additions (backward compatible)
- **Major** (X.x.x): Breaking changes, major API changes

Given this is a WASM wrapper, most releases will likely be minor or patch versions unless the underlying Rust API changes significantly.