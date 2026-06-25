# core/configuration Capability

## ADDED Requirements

### Requirement: Pixoo base URL configuration
The bridge SHALL read `PIXOO_BASE_URL` to derive the Pixoo device’s base URL when provided, sanitize it to the scheme and host for operator-facing logs, and include that sanitized host plus the configured protocol in the startup info entry so deployment tooling can confirm where the bridge is directing commands.

#### Scenario: Base URL supplied
- **WHEN** `PIXOO_BASE_URL` is set (e.g., `http://10.0.0.5`)
- **THEN** the bridge uses that host to reach the Pixoo device and logs a sanitized `pixoo_base_url` field containing the scheme and host only

#### Scenario: Base URL omitted
- **WHEN** `PIXOO_BASE_URL` is unset
- **THEN** the bridge omits the `pixoo_base_url` field from the startup log while still allowing downstream configuration helpers to build the Pixoo client without a base URL override
