## api/tools Capability

## Purpose
Expose an intuitive HTTP surface for Pixoo's timer, stopwatch, scoreboard, and soundmeter tools so automation systems can compose live-show behaviors without memorizing opaque status codes.

## ADDED Requirements

### Requirement: Timer start and stop endpoints
The bridge SHALL expose `POST /tools/timer/start` and `POST /tools/timer/stop` so clients can control the Pixoo timer with descriptive verbs while supplying `minute`/`second` only when starting a countdown.

#### Scenario: Start timer with countdown
- **WHEN** a client sends `POST /tools/timer/start` with JSON `{ "minute": M, "second": S }`
- **THEN** the bridge issues `Tools/SetTimer` with `Minute: M`, `Second: S`, `Status: 1`, retries via the shared helper, and responds with HTTP 200 (or 204) when Pixoo acknowledges the command

#### Scenario: Stop timer command
- **WHEN** a client sends `POST /tools/timer/stop`
- **THEN** the bridge issues `Tools/SetTimer` with `Status: 0`, relies on the existing retry/backoff logic, and returns HTTP 200 (or 204) once Pixoo accepts the stop signal

### Requirement: Stopwatch action endpoint
The bridge SHALL expose `POST /tools/stopwatch/{action}` where `{action}` is `start`, `stop`, or `reset`, mapping each verb to the corresponding Pixoo `Status` so clients never interact with raw numbers.

#### Scenario: Start stopwatch verb
- **WHEN** a client sends `POST /tools/stopwatch/start`
- **THEN** the bridge issues `Tools/SetStopWatch` with `Status: 1` and responds with HTTP 200 after the Pixoo client succeeds

#### Scenario: Reset stopwatch verb
- **WHEN** a client sends `POST /tools/stopwatch/reset`
- **THEN** the bridge issues `Tools/SetStopWatch` with `Status: 2` and returns HTTP 200 once retries complete

#### Scenario: Stop stopwatch verb
- **WHEN** a client sends `POST /tools/stopwatch/stop`
- **THEN** the bridge issues `Tools/SetStopWatch` with `Status: 0` and mirrors Pixoo’s success with HTTP 200

### Requirement: Scoreboard endpoint with bounded scores
The bridge SHALL expose `POST /tools/scoreboard` accepting `blue_score` and `red_score` integers between `0` and `999` and SHALL translate them to Pixoo’s `BlueScore` and `RedScore` when issuing `Tools/SetScoreBoard` so the bridge enforces safe ranges.

#### Scenario: Update scoreboard
- **WHEN** a client sends `POST /tools/scoreboard` with `blue_score` and `red_score` inside `0..=999`
- **THEN** the bridge issues `Tools/SetScoreBoard` with the same values, reuses the existing retry/backoff helper, and responds with HTTP 200 once Pixoo acknowledges the command

#### Scenario: Reject out-of-range scores
- **WHEN** a client sends `POST /tools/scoreboard` with a score outside `0..=999`
- **THEN** the bridge responds with HTTP 400 and does not dispatch any Pixoo command, preventing retries from consuming Pixoo bandwidth

### Requirement: Soundmeter action endpoint
The bridge SHALL expose `POST /tools/soundmeter/{action}` where `{action}` is `start` or `stop`, translating each verb to Pixoo’s `NoiseStatus` so callers never see the numeric status field.

#### Scenario: Start soundmeter
- **WHEN** a client sends `POST /tools/soundmeter/start`
- **THEN** the bridge issues `Tools/SetNoiseStatus` with `NoiseStatus: 1` and returns HTTP 200 after Pixoo accepts the command

#### Scenario: Stop soundmeter
- **WHEN** a client sends `POST /tools/soundmeter/stop`
- **THEN** the bridge issues `Tools/SetNoiseStatus` with `NoiseStatus: 0` and returns HTTP 200 once the Pixoo client finishes retries
