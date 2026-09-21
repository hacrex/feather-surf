# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in FeatherSurf, please report it responsibly.

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, please email security@feathersurf.dev with:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will acknowledge receipt within 48 hours and provide a resolution timeline within 7 days.

## Scope

The following are in scope for security reports:

- Memory safety issues in Rust code
- Sandbox escape or bypass in browser shells
- Privacy leaks (data sent to third parties without consent)
- Credential storage vulnerabilities
- Remote code execution
- Cross-site scripting (XSS) in browser UI
- Privilege escalation
- Denial of service attacks

## Out of Scope

- Vulnerabilities in third-party dependencies (report these to the upstream project)
- Issues requiring physical access to the user's machine
- Social engineering attacks
- Issues in experimental or disabled-by-default features

## Security Updates

Security patches will be released as soon as possible after verification. We will:

1. Confirm the vulnerability
2. Develop and test a fix
3. Release a patch version
4. Credit the reporter (unless they prefer anonymity)

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.0.x   | Yes (pre-release) |

## Security Best Practices

FeatherSurf follows these security principles:

- **Sandboxing**: Renderer processes run in restrictive sandboxes (AppContainer on Windows, namespaces on Linux, seatbelt on macOS)
- **Privacy by Default**: No telemetry, no tracking, no data collection without explicit consent
- **Memory Safety**: Rust ownership system prevents memory safety bugs
- **最小权限**: Browser runs with minimal required permissions
- **Secure Updates**: All updates are signed and verified

## Contact

- Security email: security@feathersurf.dev
- General issues: https://github.com/hacrex/feather-surf/issues
