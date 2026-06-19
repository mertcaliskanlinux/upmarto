# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Report privately via [GitHub Security Advisories](https://github.com/mertcaliskanlinux/upmarto/security/advisories/new) or email the repository maintainer if advisories are unavailable.

Include:

- Description of the vulnerability
- Steps to reproduce
- Impact assessment (data exposure, RCE, DoS, etc.)
- Affected component (backend, CLI, SDK, extension)

We aim to acknowledge reports within **72 hours** and provide a remediation timeline within **7 days** for confirmed issues.

## Security model

Upmarto is **local-first**:

- The backend binds to localhost by default (`127.0.0.1`)
- Event data is stored locally (`data/events.log`, SQLite metadata)
- No mandatory cloud telemetry

### Deployment guidance

- Do not expose the Upmarto HTTP API to untrusted networks without authentication and TLS
- Treat `.upmarto/config.json` and queue files as sensitive if payloads contain secrets
- Run the backend with least-privilege OS permissions

## Out of scope

- Social engineering
- Denial of service via extremely large local event volumes without demonstrated backend crash
- Issues in third-party IDEs (Cursor, VS Code) unrelated to Upmarto code

## Disclosure

We follow coordinated disclosure. Credit will be given in release notes unless you prefer anonymity.
