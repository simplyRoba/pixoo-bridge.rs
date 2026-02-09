## ADDED Requirements

### Requirement: Animation speed factor configuration
The bridge SHALL read `PIXOO_ANIMATION_SPEED_FACTOR` as a floating-point multiplier applied to per-frame delays read from animated files. The default SHALL be `1.4`. Invalid or non-positive values SHALL fall back to the default with a warning logged. The parsed value SHALL be stored in `AppConfig` and threaded into `AppState`.

#### Scenario: Default animation speed factor
- **WHEN** `PIXOO_ANIMATION_SPEED_FACTOR` is undefined in the environment
- **THEN** the bridge uses `1.4` as the animation speed factor

#### Scenario: Valid custom speed factor
- **WHEN** `PIXOO_ANIMATION_SPEED_FACTOR` is set to `2.0`
- **THEN** the bridge uses `2.0` as the animation speed factor and all animated frame delays are multiplied by `2.0`

#### Scenario: Invalid speed factor falls back to default
- **WHEN** `PIXOO_ANIMATION_SPEED_FACTOR` is set to `abc`
- **THEN** the bridge logs a warning naming the invalid value and falls back to `1.4`

#### Scenario: Non-positive speed factor falls back to default
- **WHEN** `PIXOO_ANIMATION_SPEED_FACTOR` is set to `0` or `-1.0`
- **THEN** the bridge logs a warning and falls back to `1.4`

### Requirement: Maximum image size configuration
The bridge SHALL read `PIXOO_BRIDGE_MAX_IMAGE_SIZE` as a human-readable byte size limiting uploaded image files. The value SHALL accept formats like `5MB`, `128KB`, `1024B` (case-insensitive, with or without the trailing `B` — e.g. `5M` and `5MB` are equivalent). The bridge SHALL use binary units (1 KB = 1024 bytes). The default SHALL be `5MB` (5,242,880 bytes). Invalid values SHALL fall back to the default with a warning logged. The parsed value SHALL be stored in `AppConfig` and threaded into `AppState`.

#### Scenario: Default max image size
- **WHEN** `PIXOO_BRIDGE_MAX_IMAGE_SIZE` is undefined in the environment
- **THEN** the bridge uses 5,242,880 bytes (5 MB) as the maximum image size

#### Scenario: Valid custom size in megabytes
- **WHEN** `PIXOO_BRIDGE_MAX_IMAGE_SIZE` is set to `10MB`
- **THEN** the bridge uses 10,485,760 bytes as the maximum image size

#### Scenario: Valid custom size in kilobytes without B suffix
- **WHEN** `PIXOO_BRIDGE_MAX_IMAGE_SIZE` is set to `128K`
- **THEN** the bridge uses 131,072 bytes as the maximum image size

#### Scenario: Invalid size value falls back to default
- **WHEN** `PIXOO_BRIDGE_MAX_IMAGE_SIZE` is set to `lots`
- **THEN** the bridge logs a warning naming the invalid value and falls back to 5,242,880 bytes
