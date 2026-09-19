# Tab Execution Security and Sandboxing Architecture

## 1. Purpose and security objective

This specification defines how FeatherSurf executes untrusted web content while
keeping the browser shell, user profile, credentials, extensions, and operating
system resources separated from compromised pages. A webpage must be treated as
hostile input even when it is displayed in a trusted window. The design assumes
that an attacker may control HTML, JavaScript, WebAssembly, downloads, redirects,
embedded frames, service workers, advertisements, and extension-facing page
content.

The primary security objective is to contain a renderer compromise. A successful
exploit in a renderer must not automatically grant access to arbitrary local
files, profile databases, credentials, other sites' storage, browser control,
or the operating system. Higher-privilege operations must be performed by a
small broker after validating an explicit, typed request.

> Sandboxing reduces the impact of a compromise; it does not make a vulnerable
> renderer trustworthy. Chromium and all platform security mechanisms must be
> kept current, and the browser must fail closed when a required boundary cannot
> be established.

## 2. Trust zones

FeatherSurf uses separate processes and explicit IPC boundaries for the following
trust zones.

| Zone | Main responsibility | Trust level | Direct access permitted |
|---|---|---|---|
| Browser shell | Window, tab selection, navigation UI, policy orchestration | High, but attack-sensitive | UI state and broker clients; no page content by default |
| Privileged broker | Validated filesystem, permission, download, update, and profile operations | Highest | Only narrowly scoped OS capabilities |
| Renderer | HTML, JavaScript, WebAssembly, DOM, site storage, frames | Untrusted | Its isolated origin data and approved browser IPC endpoints |
| GPU process | GPU command processing and compositing | Reduced privilege | GPU device and shared graphics resources only |
| Utility processes | Network, audio, storage, PDF, media, and other specialized work | Reduced privilege | Only the resources required by one utility role |
| Extension processes | Extension background workers and content scripts | Untrusted or semi-trusted | Declared extension APIs and extension-owned storage |
| Crash and telemetry worker | Opt-in diagnostics processing | Low trust | Explicitly redacted diagnostic payloads only |
| Update service | Download and signature verification of updates | High, separate from page execution | Update cache and signature keys; no page data |

The renderer and extension zones must never be treated as trusted merely because
they are running inside a FeatherSurf window. A browser-origin document and a
website-origin document are different security principals and must not share
privileged execution paths.

## 3. Process model

A browser instance consists of one browser-shell process, one or more isolated
renderer processes, and separate utility processes. The exact Chromium process
layout may vary by platform, but the security invariants do not.

```text
User interface process
        │ typed, authenticated IPC
        ▼
Capability broker ─────── Update verifier
   │       │  \
   │       │   └── Permission service
   │       └────── Download service
   └────────────── Profile/storage service
        │
        ├── Renderer process: site instance A
        ├── Renderer process: site instance B
        ├── Extension process: extension X
        ├── GPU process
        └── Sandboxed utility processes
```

### 3.1 Browser shell

The browser shell owns windows, tab identity, user-visible navigation state, and
policy decisions. It may request a broker operation, but it must not grant itself
unrestricted filesystem or credential access merely because a page asked for an
operation. Page-originated requests must carry the originating tab, frame, origin,
permission context, and user-gesture information needed for policy evaluation.

The shell must not inject raw page strings into privileged command lines, paths,
SQL statements, logs, or structured IPC without validation and length limits.

### 3.2 Renderer processes

Each renderer runs with the strongest available platform sandbox. Renderer code
must not have direct access to the user profile directory, credential store,
update keys, arbitrary sockets, device nodes, or unrestricted child-process
creation. Renderer-to-browser communication must use typed messages and an
allowlisted interface.

Renderer processes are grouped by site instance or equivalent Chromium isolation
policy. Cross-origin documents should not share a renderer when site isolation is
available. Cross-origin frames remain separate security principals even when the
engine places them in the same process for a platform-specific reason.

### 3.3 Broker process

The broker is the only component allowed to perform privileged operations on
behalf of untrusted content. It should have a small API surface and no page
rendering responsibilities. Every method must specify:

- The caller identity and process channel.
- The tab, frame, profile, origin, and extension identity.
- The required user permission.
- Whether a transient user gesture is mandatory.
- Input length, encoding, and schema limits.
- The permitted output data and redaction rules.
- Audit events that may be recorded without page content.

The broker must reject unknown message types, unknown fields when strict parsing
is enabled, invalid enum values, stale request tokens, requests from dead tabs,
and requests whose origin or profile does not match the registered channel.

## 4. Capability-based access model

FeatherSurf should use capability tokens rather than ambient authority. A token
is a short-lived, unguessable reference to one narrowly scoped permission. It is
bound to the browser instance, process channel, tab, frame, origin, profile,
operation, and expiry time.

A capability must not be transferable through ordinary page strings or exposed to
another origin. The broker must check the binding on every use rather than trusting
that a token was validated when it was created.

Examples include:

- A file chooser capability bound to one user-approved upload operation.
- A download capability bound to one URL, destination policy, and tab.
- A notification capability bound to one origin and profile.
- A microphone capability bound to one origin, device policy, and session.
- A clipboard-write capability requiring a user gesture.
- A developer-tools capability available only to an explicit developer surface.

Capabilities must expire on navigation, origin change, tab closure, profile lock,
permission revocation, or browser shutdown unless the operation explicitly
requires persistence and the user has granted it.

## 5. Site, origin, and profile isolation

### 5.1 Origin isolation

Cookies, local storage, IndexedDB, service-worker registrations, cache entries,
permissions, and credentials must be keyed by the browser profile and origin.
The scheme, host, port, and storage partition must be included in the key. A
string comparison of hostnames alone is insufficient.

Navigation from one origin to another must invalidate origin-bound capabilities
and page-provided references. Redirects must be evaluated as new origins where
required, not treated as a continuation of the initial trust decision.

### 5.2 Site isolation

The engine integration must enable the strongest supported site-isolation mode.
Cross-origin frames must use the engine's origin checks, process isolation, and
postMessage rules. FeatherSurf must not add a custom shortcut that exposes shell
objects directly to every frame.

Privileged browser pages must use a separate browser-owned scheme or equivalent
isolated mechanism. They must apply strict content security policy, avoid unsafe
HTML construction, validate messages from web content, and never expose a generic
`eval`, filesystem, shell, or arbitrary IPC bridge.

### 5.3 Profile and container isolation

Profiles and containers are separate security domains for cookies, storage,
history, permissions, extensions, downloads, and credentials. A tab may not
request a resource from another profile by changing a path, origin string, or
IPC field. Profile IDs must be assigned by the browser shell and looked up by
opaque internal identifiers.

Profile databases must be opened by the profile service, not by renderers. The
service must apply file permissions, locking, atomic writes, migration checks,
and corruption recovery. Secrets must be stored in the operating-system
credential facility or an equivalent protected vault, never in page-readable
files.

## 6. IPC security

### 6.1 Channel establishment

Every renderer and utility channel must be registered by the browser shell with:

- A process ID and authenticated channel identity.
- A tab and frame identity where applicable.
- A profile and origin context.
- A process role.
- A negotiated protocol version.
- A monotonic request sequence or replay-protection value.

The broker must not accept a claimed identity from a renderer message as proof of
identity. Identity comes from the authenticated channel registration.

### 6.2 Message validation

IPC messages must use versioned schemas with bounded fields. The broker must:

1. Decode using strict types and reject malformed encodings.
2. Enforce maximum message size and per-channel rate limits.
3. Validate that referenced tabs, frames, origins, profiles, and capabilities exist.
4. Check that the operation is allowed for the caller's process role.
5. Check user permission and gesture requirements.
6. Perform the operation using safe APIs.
7. Return only the minimum necessary result.
8. Record a privacy-safe decision event when diagnostics are enabled.

A renderer must not be able to cause a privileged operation by sending an IPC
message that merely contains a URL, path, profile ID, or permission name. The
broker must derive or cross-check these values from its registered context.

### 6.3 Failure and cancellation

Requests must be cancellable when a tab navigates, closes, loses permission, or
changes origin. A late response must not be applied to a new document that reused
the same tab ID. Each request should carry a document-generation identifier in
addition to the stable tab ID.

When validation or authorization fails, the broker returns a generic denial and
avoids revealing whether a protected path, profile, device, or credential exists.
Repeated failures should be rate-limited without exposing a side channel through
large timing differences.

## 7. Filesystem, network, and device boundaries

### 7.1 Filesystem

Renderers have no arbitrary filesystem access. File uploads use a user-selected
handle or broker-managed temporary copy. The broker canonicalizes and validates
paths, rejects traversal and ambiguous encodings, restricts destinations to an
approved directory, and uses safe creation flags to avoid symlink attacks.

Downloads are written by the download service, not by renderer code. The service
must prevent path traversal, enforce quotas, handle name collisions safely, and
avoid executing downloaded files automatically. Temporary files must be created
with restrictive permissions and removed after cancellation or failure.

### 7.2 Network

The network service applies origin, proxy, certificate, cookie, and privacy
policy before dispatch. Renderer requests must not bypass the browser's proxy,
certificate validation, tracker policy, or private-network restrictions through a
second raw socket path.

DNS, proxy, certificate, and connection errors should not expose local network
inventory beyond what the web platform explicitly permits. Enterprise or user
configuration may change the policy, but the effective policy must be inspectable
in the permission and diagnostics UI.

### 7.3 Devices

Camera, microphone, location, notifications, USB, Bluetooth, serial, MIDI,
clipboard, and file-system access require explicit permission checks. Device
handles are scoped to the origin and session. Revocation must invalidate active
handles where the platform permits it.

The GPU process and media utilities receive only the device access needed for
their roles. A renderer must not receive a raw device handle as a substitute for
an origin-bound browser permission.

## 8. Extensions and privileged browser pages

Extensions are separate principals from websites. Content scripts inherit the
page's DOM exposure but must not inherit arbitrary browser-shell authority.
Background workers run in extension-specific processes or equivalent isolated
contexts. Extension permissions are declared, displayed, and enforced by the
browser.

Extension storage, network access, downloads, native messaging, and debugging
must be independently permissioned. A compromised extension must not become a
universal bypass for site isolation or profile boundaries.

Browser-owned pages such as settings, downloads, history, and the developer
center must not use a broad JavaScript bridge. They should expose narrow commands
with origin checks, strict schemas, CSRF-resistant state tokens, and content
security policies that prohibit inline script and untrusted script sources.

## 9. Memory-management interaction with security

The eviction manager is a policy component, not a security boundary. It must not
assume that a tab's score, title, URL, process label, or renderer-reported state
is trustworthy. A compromised renderer can emit false activity or memory data.

Security-relevant rules are therefore hard guards outside the score:

- Protected permissions and active user prompts cannot be silently discarded.
- A tab with an active permission request must not lose the request's security
  context during suspension.
- A snapshot must not contain raw credentials, capability tokens, or unredacted
  secrets unless the protected storage policy explicitly allows it.
- Restore operations must create a new document-generation context and recheck
  origin-bound permissions.
- A discarded renderer must not retain a valid privileged IPC capability after
  process termination.
- Memory pressure must never justify bypassing sandbox, origin, or permission
  checks.

Before suspension, the browser adapter must serialize only approved state. On
restore, it must treat all renderer-provided state as untrusted input and apply
normal navigation, origin, and permission checks again.

## 10. Compromise and recovery model

### 10.1 Renderer compromise

If a renderer is suspected of compromise, the browser should terminate the
renderer process, revoke its channel capabilities, invalidate its document
context, preserve only safe session metadata, and create a fresh renderer on
restore. The browser shell must remain usable and must not execute renderer
requests during teardown.

### 10.2 Utility or GPU compromise

Utility and GPU processes receive narrower capabilities than the browser shell.
A crash or compromise must cause channel revocation and process replacement. The
browser should degrade functionality rather than grant the process additional
privileges to recover.

### 10.3 Broker failure

If the broker cannot validate or authorize a request, the default result is
rejection. If the broker exits, the browser should close or disable privileged
operations until a fresh authenticated broker is established. It must not fall
back to direct renderer filesystem, network, device, or shell access.

### 10.4 Profile corruption

Profile recovery must use an atomic snapshot or backup. The browser must not
attempt to execute content from a corrupt database as code or silently substitute
another profile. Recovery should preserve user data where possible while
requiring explicit consent for destructive repair.

## 11. Platform sandbox requirements

The platform adapter must enable the strongest supported sandbox primitives and
make their status observable in diagnostics.

### Linux

- Use Chromium's namespace, seccomp, setuid or user-namespace, and capability
  restrictions according to the selected integration.
- Apply `no_new_privs` and restrict filesystem namespaces for renderer and utility
  processes.
- Use a compositor/GPU policy that does not expose unnecessary device nodes.
- Prefer a dedicated, permission-restricted profile directory.
- Fail closed if the required sandbox cannot initialize unless the user selects
  an explicitly labeled unsafe development mode.

### Windows

- Use AppContainer or equivalent renderer isolation where supported.
- Apply restricted tokens, job objects, integrity levels, and mitigations such as
  Control Flow Guard according to the engine requirements.
- Keep browser profile and credential directories outside renderer access.
- Ensure child-process creation is denied or brokered for renderer roles.

### macOS

- Use the engine's renderer sandbox and seatbelt profile.
- Apply hardened runtime and appropriate entitlements.
- Keep keychain access in a brokered, user-approved service.
- Do not grant broad file entitlements to the renderer or extension process.

The exact mechanism is platform-specific, but the acceptance condition is common:
renderer code must not obtain arbitrary local authority by default, and the
browser must report when a required sandbox feature is unavailable.

## 12. Security invariants

The following invariants must be tested and preserved across refactors:

1. A renderer cannot read another origin's cookies, storage, credentials, or
   profile files.
2. A renderer cannot create an arbitrary process or socket.
3. A renderer cannot invoke a privileged operation without a valid, bound,
   unexpired capability and any required user gesture.
4. Navigation invalidates document-bound capabilities.
5. Closing or killing a renderer revokes all of its capabilities.
6. A tab ID alone is never sufficient to authorize a response or operation.
7. Memory pressure never bypasses security checks.
8. Download and upload paths cannot escape their broker-approved directories.
9. Browser-owned pages do not expose generic shell or filesystem bridges.
10. A sandbox initialization failure is visible and does not silently become a
    production-quality unsandboxed execution mode.

## 13. Verification plan

### Unit tests

- Validate every IPC schema and enum boundary.
- Test capability binding, expiry, revocation, and replay rejection.
- Test origin and profile-key construction.
- Test safe path handling and symlink resistance.
- Test snapshot redaction and document-generation checks.
- Test permission denial and user-gesture requirements.

### Integration tests

- Start a renderer with the expected sandbox and verify its process restrictions.
- Navigate across origins and verify capability invalidation.
- Attempt cross-profile storage access.
- Attempt path traversal through downloads and uploads.
- Kill a renderer and verify channel and capability revocation.
- Suspend and restore a tab while a permission request is pending.
- Verify that a high-memory-pressure eviction cannot discard a protected tab.
- Verify that browser-owned pages reject messages from ordinary web origins.

### Adversarial tests

- Fuzz every broker message decoder.
- Fuzz navigation, redirect, and restore sequences.
- Fuzz malformed snapshot data.
- Run extension and page content with hostile IPC payloads.
- Test oversized messages and request floods.
- Test process churn, renderer crashes, broker restarts, and low-memory events.
- Run sanitizer and memory-safety checks where supported by the toolchain.

### Release gates

A release must not ship if the renderer sandbox is disabled unexpectedly, a
privileged IPC endpoint accepts unvalidated caller-controlled identity, a
credential or capability appears in a snapshot or diagnostic bundle, or a high-
severity cross-origin, profile-isolation, or update-verification defect remains
unresolved.

## 14. Operational requirements

Security-sensitive decisions should emit structured, privacy-safe events such as
`permission_denied`, `capability_revoked`, `renderer_terminated`, and
`sandbox_unavailable`. Events must not include page contents, credentials, full
URLs, form values, or capability tokens. Crash reports and telemetry remain
opt-in, and the default diagnostic path should retain data locally until the user
chooses to export it.

The Chromium version, sandbox status, process roles, enabled site-isolation mode,
and security-policy version should be visible in a diagnostics view. This makes
security assumptions inspectable without exposing sensitive browsing data.

## 15. Implementation order

1. Select the Chromium integration and document its sandbox guarantees.
2. Implement authenticated, typed renderer-to-browser IPC.
3. Implement the capability broker with profile and origin binding.
4. Enforce process and filesystem isolation before adding convenience APIs.
5. Add site isolation, permission, download, and profile integration tests.
6. Add renderer-crash recovery and capability revocation.
7. Add platform-specific sandbox verification to CI and release checks.
8. Conduct an independent security review before beta distribution.
