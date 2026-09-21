# Security Model

This document defines FeatherSurf's threat model, security boundaries, and
privacy guarantees. It complements `tab-execution-security.md` (which covers
renderer sandboxing) by addressing the full attack surface.

## 1. Threat model

### 1.1 Attacker capabilities

We assume an attacker may:

- Control arbitrary web content (HTML, JS, WASM, CSS, images, media)
- Control embedded iframes, redirects, service workers, and ads
- Control extension-facing page content and messaging
- Observe network traffic between the browser and servers
- Control a malicious extension (if installed)
- Control a compromised renderer process
- Control a malicious update server (supply-chain attack)
- Control a malicious AI provider (if AI is enabled)
- Physical access to the device (for credential theft)

### 1.2 Attack surfaces

| Surface | Description | Risk level |
|---|---|---|
| **Web content** | Untrusted HTML/JS/WASM from any origin | Critical |
| **Extensions** | Semi-trusted code with declared permissions | High |
| **Renderer compromise** | Exploited renderer process | Critical |
| **Network observers** | MITM, ISP, public Wi-Fi | High |
| **Local profile theft** | Physical access or malware reading profile files | High |
| **Supply-chain attacks** | Malicious updates, compromised dependencies | High |
| **AI providers** | Cloud/local AI receiving page data | Medium |
| **Sync server** | Server-side compromise of encrypted sync data | Medium |
| **Malicious downloads** | Files with path traversal or executable payloads | Medium |
| **Clipboard snooping** | Other apps reading clipboard contents | Low |

### 1.3 Trust boundaries

```
┌─────────────────────────────────────────────────────┐
│                    Untrusted Zone                    │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐            │
│  │Website A│  │Website B│  │  Ads    │            │
│  └────┬────┘  └────┬────┘  └────┬────┘            │
│       │            │            │                   │
│  ─────┼────────────┼────────────┼──── IPC ──────── │
│       │            │            │                   │
├───────┼────────────┼────────────┼───────────────────┤
│       ▼            ▼            ▼   Broker Zone     │
│  ┌─────────────────────────────────────────┐       │
│  │           Capability Broker              │       │
│  │  - Validates every request               │       │
│  │  - Checks permissions + user gesture     │       │
│  │  - Enforces origin + profile binding     │       │
│  └─────────────────┬───────────────────────┘       │
│                    │                                │
├────────────────────┼────────────────────────────────┤
│                    ▼    Trusted Zone                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │ Profile  │ │Credentials│ │  Sync    │           │
│  │ Storage  │ │  Store   │ │  Keys    │           │
│  └──────────┘ └──────────┘ └──────────┘           │
└─────────────────────────────────────────────────────┘
```

## 2. Profile and credential boundaries

### 2.1 Profile isolation

Each profile is a separate security domain:

- Cookies, localStorage, IndexedDB are keyed by `(profile_id, origin)`
- History, bookmarks, and settings are stored per-profile
- Extensions are installed per-profile
- Downloads are routed per-profile
- Permissions are granted per-profile

Cross-profile access is forbidden:

- A renderer cannot request resources from another profile
- Profile IDs are assigned by the shell, not by renderer content
- Profile databases are opened by the profile service, not by renderers

### 2.2 Credential storage

| Platform | Storage mechanism | Encryption |
|---|---|---|
| Windows | Windows Credential Manager / DPAPI | AES-256, user password |
| Linux | `libsecret` / `gnome-keyring` | User keyring passphrase |
| macOS | Keychain Services | AES-256, user password + Secure Enclave |
| Android | Android Keystore | Hardware-backed AES-256 |
| iOS | iOS Keychain | AES-256, device passcode + Secure Enclave |

Rules:

- Credentials are never stored in plaintext application files
- The browser never has direct access to raw key material
- All credential operations go through the OS credential API
- Profile encryption keys are derived from the OS credential store
- Sync keys are generated per-device and never leave the device unencrypted

### 2.3 Session data

Session files contain:

- Open tab URLs, titles, scroll positions
- Navigation history
- Active tab selection

Session files do NOT contain:

- Passwords or passkeys
- Cookies (stored separately by the engine)
- Capability tokens
- Sync keys
- Form state with sensitive data (SSN, credit cards)

## 3. Privacy guarantees

### 3.1 Default privacy settings

| Setting | Default | User can change |
|---|---|---|
| Third-party tracker blocking | ON | Yes |
| Known malicious domain blocking | ON | No (security hardening) |
| Fingerprinting protection | ON | Yes (per-site exceptions) |
| Third-party cookie restriction | ON | Yes (per-site exceptions) |
| Telemetry | OFF | Yes (opt-in) |
| Crash reporting | OFF | Yes (opt-in) |
| AI data collection | OFF | Yes (per-action consent) |
| DNS-over-HTTPS | ON (where available) | Yes |

### 3.2 Telemetry data flow

When telemetry is OFF (default):

- No data leaves the device
- No network requests are made for telemetry
- Crash reports are stored locally only

When telemetry is ON (opt-in):

- Data sent: browser version, OS version, crash stack traces
- Data NOT sent: URLs, page content, form data, credentials, browsing history
- Data is anonymized with a rotating device identifier
- User can view and delete all collected data

### 3.3 Crash reporting data flow

When crash reporting is OFF (default):

- Minidumps are stored in the profile directory
- User can manually export and submit

When crash reporting is ON (opt-in):

- Minidumps are sent to the crash server
- URLs are redacted from crash reports
- Form data is never included
- Stack traces are symbolized locally before sending

## 4. Network security

### 4.1 HTTPS enforcement

- All navigation defaults to HTTPS
- HTTP is downgraded to HTTPS where possible (HSTS preload list)
- Certificate errors are not bypassable (no "proceed anyway" for known-bad certs)

### 4.2 DNS-over-HTTPS

- Enabled by default where supported
- Uses Cloudflare or Google DNS (user-configurable)
- Fallback to system DNS if DoH fails

### 4.3 Certificate validation

- Full certificate chain validation
- OCSP stapling support
- Certificate Transparency enforcement
- No custom certificate authorities (unless user explicitly installs)

## 5. Update security

### 5.1 Signed updates

- All updates are signed with a dedicated signing key
- Package signing key ≠ update signing key (separation of duties)
- Signatures are verified before installation
- Rollback is prevented (monotonically increasing version numbers)

### 5.2 Update transport

- Downloads use HTTPS only
- Checksums (SHA-256) are verified after download
- Staged rollouts limit blast radius of compromised updates

### 5.3 Emergency revocation

- Signing keys can be rotated in case of compromise
- Emergency updates can force key rotation
- Users are notified of key rotation events

## 6. Extension security

### 6.1 Permission model

- Extensions declare permissions in manifest
- Permissions are displayed to the user during installation
- Runtime permission checks enforce declared permissions
- Extensions cannot escalate permissions at runtime

### 6.2 Isolation

- Extension processes are separate from renderer processes
- Extension storage is isolated per-profile
- Content scripts run in the page's context but with limited API access
- Background workers run in isolated contexts

### 6.3 Compromise containment

- A compromised extension cannot access other profiles
- A compromised extension cannot access the credential store
- A compromised extension cannot modify browser settings
- A compromised extension's network access is restricted to declared hosts

## 7. AI security

### 7.1 Data minimization

- AI is disabled by default (zero RAM overhead)
- When enabled, only explicitly selected text is sent to the AI provider
- Page URLs are sent only if the user explicitly opts in
- No background data collection for AI training

### 7.2 Provider isolation

- Each AI request is authenticated and scoped
- Requests include the minimum necessary context
- Provider responses are sandboxed from the browser shell
- Local AI models run in isolated processes

### 7.3 Prompt injection defense

- Page content is never injected into AI prompts directly
- User-selected text is the only input to AI features
- AI responses are treated as untrusted content
- No execution of AI-generated code without user confirmation

## 8. Memory management security

### 8.1 Eviction is not a security boundary

The eviction manager makes policy decisions, not security decisions. It must not:

- Assume renderer-reported data is trustworthy
- Bypass sandbox, origin, or permission checks
- Discard tabs with active permission requests
- Expose credentials or tokens in snapshots

### 8.2 Snapshot safety

Before suspension, the browser serializes only:

- URL, title, scroll position
- Non-sensitive form state (text fields, checkboxes)
- Navigation history

Snapshots never contain:

- Passwords or credit card numbers
- Capability tokens
- Session cookies
- Sync keys
- Raw page content

### 8.3 Restore safety

On restore:

- All renderer-provided state is treated as untrusted
- Origin-bound permissions are rechecked
- Navigation and origin checks are reapplied
- A new document-generation context is created

## 9. Platform-specific security

### 9.1 Windows

- AppContainer sandbox for renderer processes
- Control Flow Guard (CFG) for code integrity
- Restricted tokens and job objects
- Windows Defender SmartScreen integration for downloads
- DPAPI for credential encryption

### 9.2 Linux

- User namespaces for renderer sandboxing
- seccomp-bpf for syscall filtering
- `no_new_privs` flag
- Filesystem namespace restrictions
- `libsecret` for credential storage

### 9.3 macOS

- App sandbox with entitlements
- Hardened runtime
- Seatbelt profiles for renderer
- Keychain Services for credentials
- Gatekeeper integration for downloads

### 9.4 Android

- Android sandbox (per-app UID)
- Runtime permission model
- Android Keystore for credentials
- Network security config for TLS
- Play Protect integration

### 9.5 iOS

- App sandbox (per-app container)
- WKWebView sandbox (WebKit enforced)
- iOS Keychain for credentials
- App Transport Security (ATS)
- No extension support (WebKit limitation)

## 10. Security invariants

These invariants must hold at all times and be tested:

1. A renderer cannot read another origin's cookies, storage, or credentials
2. A renderer cannot create an arbitrary process or socket
3. A renderer cannot invoke a privileged operation without a valid capability
4. Navigation invalidates document-bound capabilities
5. Killing a renderer revokes all its capabilities
6. A tab ID alone is never sufficient to authorize an operation
7. Memory pressure never bypasses security checks
8. Download paths cannot escape broker-approved directories
9. Browser pages do not expose generic shell or filesystem bridges
10. Sandbox failures are visible and never silently accepted in production

## 11. Security review requirements

The following changes require a security review before merging:

- Any IPC schema change
- Any permission check modification
- Any credential storage change
- Any update verification change
- Any snapshot serialization change
- Any extension API change
- Any AI data flow change
- Any privacy setting default change
- Any sandbox configuration change

## 12. Verification plan

### Unit tests

- IPC schema validation and boundary testing
- Capability binding, expiry, revocation
- Origin and profile key construction
- Safe path handling (traversal, symlink resistance)
- Snapshot redaction verification

### Integration tests

- Cross-origin capability invalidation
- Cross-profile storage isolation
- Renderer crash recovery and capability revocation
- Suspended tab restoration with pending permission requests
- Download path escape attempts

### Adversarial tests

- Fuzz all broker message decoders
- Fuzz navigation, redirect, and restore sequences
- Fuzz malformed snapshot data
- Extension and page content with hostile IPC payloads
- Oversized messages and request floods
- Process churn and low-memory events
