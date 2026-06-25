## MODIFIED Requirements

### Requirement: Startup logging records runtime configuration
The bridge SHALL emit an info-level log once at startup that lists the resolved `health_forward` flag, the Pixoo base URL, the listener address, the binary version, the `animation_speed_factor`, and the `max_image_size` so operators know what settings the container began with and which artifact they deployed.

#### Scenario: Container starts with health forwarding enabled
- **WHEN** the service finishes building `AppState` or equivalent and before it accepts HTTP traffic
- **THEN** it logs an info entry containing `health_forward=true`, the configured base URL, the listener address, the resolved `animation_speed_factor`, and the configured `max_image_size`
