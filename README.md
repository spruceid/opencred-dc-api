# DC API

A Rust-based Digital Credential API implementation providing OpenID4VP (OpenID for Verifiable Presentations) client functionality with WebAssembly support for Node.js/TypeScript environments.

## Overview

DC API is a WebAssembly-based library that enables digital credential operations with full OpenID4VP protocol support. It provides session management, credential request/response handling, and X.509 certificate-based authentication.

**Key Features:**
- OpenID4VP verifier client implementation
- X.509 certificate chain validation and SAN-based client identification
- Session-based credential request/response flow
- WebAssembly bindings for Node.js/TypeScript
- HPKE encryption support (P-256)
- ISO/IEC 18013-5 mDL (mobile Driver's License) integration

## Repository Structure

```
dc-api/
├── core/           # Core Rust library with DC API functionality
├── wasm/           # WebAssembly bindings via wasm-bindgen
├── npm-package/    # Node.js/TypeScript package
└── .github/        # CI/CD workflows
```

## Quick Start

### Using the npm Package

Install the package:

```bash
npm install @spruceid/opencred-dc-api
```

Use in your Node.js/TypeScript project:

```typescript
import { DcApi, JsOid4VpSessionStore } from '@spruceid/opencred-dc-api';

const oid4vpStore = JsOid4VpSessionStore.createMemoryStore();
const dcApiStore = /* implement DcApiSessionStore interface */;

const dcApi = await DcApi.new(
  privateKeyPem,        // PKCS8 PEM encoded private key
  'https://api.example.com',
  'https://api.example.com/submit',
  'https://api.example.com/reference',
  certChainPem,
  oid4vpStore,
  dcApiStore
);

const session = await dcApi.create_new_session();
const result = await dcApi.initiate_request(session.id, session.secret, request);
```

See the [npm-package README](npm-package/README.md) for complete documentation.

### Building from Source

#### Prerequisites

- Rust toolchain (stable)
- `wasm32-unknown-unknown` target
- wasm-bindgen-cli (v0.2.101)
- Node.js 16+

#### Build Steps

1. **Build the WASM package:**
   ```bash
   cd npm-package
   npm install
   npm run build
   ```

2. **Run Rust tests:**
   ```bash
   cargo test --workspace
   ```

3. **Run linting:**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-features -- -D warnings
   ```

## Core Components

### DC API Core (`core/`)

The core Rust library implementing:

- **Client** (`client.rs`): OID4VP client with X.509 certificate-based authentication
- **X.509 Client** (`x509_client.rs`): X.509 certificate chain handling and SAN variant support
- **Session Management** (`session.rs`): Session storage and lifecycle management
- **Configuration** (`config.rs`): Configuration structures for OID4VP client setup
- **Annex C & D**: ISO/IEC 18013-5 protocol implementations
- **Types** (`types.rs`): Core data structures and type definitions

### WebAssembly Bindings (`wasm/`)

WASM bindings generated via wasm-bindgen that expose:
- `DcApi`: Main API class for credential operations
- `JsOid4VpSessionStore`: Session storage with JavaScript interop
- `JsDcApiSessionDriver`: Internal session management driver

### npm Package (`npm-package/`)

TypeScript/Node.js wrapper providing:
- Type definitions for all WASM-exported classes
- Session store interfaces
- Build scripts and CI/CD integration
- Comprehensive API documentation

## Development

### Running Tests

```bash
# Rust tests
cargo test --workspace

# npm package tests (includes WASM build)
cd npm-package
npm test
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-features -- -D warnings

# Security audit
cargo audit
cd npm-package && npm audit
```

## CI/CD

The repository includes automated workflows:

- **CI** (`ci.yml`): Runs on every push/PR
  - WASM build validation
  - Package structure verification
  - Integration tests
  - Rust tests, formatting, and Clippy
  - Security audits (npm + Cargo)

- **CD** (`cd.yml`): Triggered by git tags (`npm/v*`)
  - Builds and publishes to npm
  - Validates version numbers
  - Automated release process

## Release Process

Create a release by tagging:

```bash
cd npm-package
npm run release:patch  # or release:minor, release:major
git push origin npm/v$(node -p "require('./package.json').version")
```

See [npm-package/RELEASE.md](npm-package/RELEASE.md) for detailed instructions.

## Dependencies

### Core Rust Dependencies
- `openid4vp`: OpenID4VP protocol implementation
- `isomdl`: ISO/IEC 18013-5 mobile Driver's License
- `x509-cert`: X.509 certificate handling
- `coset`: CBOR Object Signing and Encryption
- `hpke`: Hybrid Public Key Encryption (P-256)
- `p256`: NIST P-256 elliptic curve

### WASM Dependencies
- `wasm-bindgen`: Rust/JavaScript interop
- `web-sys`, `js-sys`: Web platform bindings

## License

This project is dual-licensed under:
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

You may choose either license for your use.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure CI passes (`cargo test`, `cargo clippy`, `cargo fmt`)
5. Submit a pull request

## Support

- **Issues**: [GitHub Issues](https://github.com/spruceid/dc-api/issues)
- **Documentation**: See [npm-package/README.md](npm-package/README.md) for API docs
- **Examples**: Check `npm-package/tests/` for usage examples

## Related Projects

- [openid4vp](https://github.com/spruceid/openid4vp): OpenID for Verifiable Presentations
- [isomdl](https://github.com/spruceid/isomdl): ISO mobile Driver's License implementation
