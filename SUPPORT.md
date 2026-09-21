# Support

## Getting Help

FeatherSurf is in active development. Here's where to get help:

### Documentation

- **Architecture**: See `docs/` directory for ADRs and design documents
- **API Reference**: Run `cargo doc --workspace` in the `rust/` directory
- **TODO**: See `todo.md` for current project status and roadmap

### Questions and Discussions

- **GitHub Discussions**: https://github.com/hacrex/feather-surf/discussions
- **General Questions**: Open a discussion with the "question" label

### Bug Reports

- **Bug Reports**: https://github.com/hacrex/feather-surf/issues/new?template=bug_report.md
- **Feature Requests**: https://github.com/hacrex/feather-surf/issues/new?template=feature_request.md

### Development

- **Contributing**: See CONTRIBUTING.md for development workflow
- **Good First Issues**: https://github.com/hacrex/feather-surf/labels/good%20first%20issue
- **Architecture Decisions**: See `docs/adr-*.md` files

### Security Issues

- **Security Reports**: See SECURITY.md for vulnerability reporting instructions
- **Do NOT** open public issues for security vulnerabilities

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows 10+ | Tier 1 | Primary development platform |
| Ubuntu 22.04+ | Tier 1 | Primary development platform |
| Fedora 38+ | Tier 1 | Primary development platform |
| macOS 13+ | Tier 2 | Community support |
| Debian 12+ | Tier 2 | Community support |
| Arch Linux | Tier 2 | Community support |
| Android 10+ | Tier 3 | Planned for later phase |
| iOS 16+ | Tier 3 | Planned for later phase |

## Current Status

FeatherSurf is in **pre-alpha** status (version 0.0.1). The following features are implemented:

### Completed
- Tab lifecycle state machine
- Memory scoring model
- Session persistence
- Profile isolation
- Bookmarks and history storage
- Process monitoring
- Resource monitoring
- Windows and Linux browser shells
- CEF integration (skeleton)

### In Progress
- AI integration for tab scoring
- Performance tuning
- Quality assurance

### Not Yet Implemented
- macOS browser shell
- Android browser shell
- iOS browser shell
- Full extension support
- Sync functionality
- Distribution packaging

## Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test --workspace`
5. Run lints: `cargo clippy --workspace --all-targets -- -D warnings`
6. Format code: `cargo fmt --all`
7. Submit a pull request

## Code of Conduct

See CODE_OF_CONDUCT.md for community guidelines.

## License

Apache License, Version 2.0. See LICENSE for details.
