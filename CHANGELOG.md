# Changelog

## [0.5.2](https://github.com/simplyRoba/pixoo-bridge.rs/compare/v0.5.1...v0.5.2) (2026-02-01)


### Bug Fixes

* split release build and fix wrong actions call ([3acc6f1](https://github.com/simplyRoba/pixoo-bridge.rs/commit/3acc6f143279d07032258d3c3e32867487647a91))

## [0.5.1](https://github.com/simplyRoba/pixoo-bridge.rs/compare/v0.5.0...v0.5.1) (2026-02-01)


### Bug Fixes

* **ci:** bump actions/download-artifact from 4 to 7 ([cbe24e1](https://github.com/simplyRoba/pixoo-bridge.rs/commit/cbe24e1150ee7109c705385ab64aba8e5f530203))
* **ci:** bump actions/upload-artifact from 4 to 6 ([9e6596b](https://github.com/simplyRoba/pixoo-bridge.rs/commit/9e6596b7681cb02ba9cd0950468e14b9f983a796))
* update upload-artifact action to v4 in publish-release workflow ([0f696a2](https://github.com/simplyRoba/pixoo-bridge.rs/commit/0f696a2caeee13d92641826fde909a1edfefae4b))

## [0.5.0](https://github.com/simplyRoba/pixoo-bridge.rs/compare/v0.4.0...v0.5.0) (2026-02-01)


### Features

* add GitHub Actions extension to VSCode customizations ([ac64335](https://github.com/simplyRoba/pixoo-bridge.rs/commit/ac64335ff0e9ee7e6366f4990905533a71dd773a))
* enhance github ations by splitting up jobs also move release compilation to the action ([#40](https://github.com/simplyRoba/pixoo-bridge.rs/issues/40)) ([8e24513](https://github.com/simplyRoba/pixoo-bridge.rs/commit/8e245139555b10f4931c6a33f14d9309fa445026))

## [0.4.0](https://github.com/simplyRoba/pixoo-bridge.rs/compare/v0.3.0...v0.4.0) (2026-01-31)


### Features

* add configurable startup and error logging ([#36](https://github.com/simplyRoba/pixoo-bridge.rs/issues/36)) ([c7a7b01](https://github.com/simplyRoba/pixoo-bridge.rs/commit/c7a7b0196bae6c30c4dcd364a8131154f0025185))
* add Dockerfile and devcontainer configuration for development environment ([0f8ad6f](https://github.com/simplyRoba/pixoo-bridge.rs/commit/0f8ad6fe7f37f6837a4b8acfdf6839f56e90e94c))
* add purpose sections to multiple capability specs for clarity ([bc8ef9c](https://github.com/simplyRoba/pixoo-bridge.rs/commit/bc8ef9c6c4b66fd8b49e62e7f9937e73468213c9))
* add reboot endpoint ([#38](https://github.com/simplyRoba/pixoo-bridge.rs/issues/38)) ([d5bb267](https://github.com/simplyRoba/pixoo-bridge.rs/commit/d5bb267844033722ae68e73a59028725e8c81e7c))
* add VSCode settings for default panel location ([e6467ae](https://github.com/simplyRoba/pixoo-bridge.rs/commit/e6467ae89f132776f131a2e9c6b8b17f4452b02c))
* introduce configurable HTTP listener port via PIXOO_BRIDGE_PORT ([5bd519e](https://github.com/simplyRoba/pixoo-bridge.rs/commit/5bd519ed9a52b937a946e1cfec4f4cbde1871e83))


### Bug Fixes

* standardize Dockerfile stage names to uppercase ([815dcc4](https://github.com/simplyRoba/pixoo-bridge.rs/commit/815dcc416ddee35ff1cc4d7181c56a6975586465))
* update command syntax for opsx commands and increment version to 1.1.1 ([fd2407d](https://github.com/simplyRoba/pixoo-bridge.rs/commit/fd2407d6a4c5a7014a833cf97fb545d99cf65f5d))

## [0.3.0](https://github.com/simplyRoba/pixoo-bridge.rs/compare/v0.2.4...v0.3.0) (2026-01-27)


### Features

* add dependabot config ([#22](https://github.com/simplyRoba/pixoo-bridge.rs/issues/22)) ([98aae06](https://github.com/simplyRoba/pixoo-bridge.rs/commit/98aae06673bae08438d6810a51f163d58e0ce064))
* Add new skills for OpenSpec workflow management ([#29](https://github.com/simplyRoba/pixoo-bridge.rs/issues/29)) ([dbdecf3](https://github.com/simplyRoba/pixoo-bridge.rs/commit/dbdecf34f298c9fe56cb7fdbb84b3b12afb2cd1c))
* **core:** define core domain for Pixoo bridge with HTTP client and Rust foundation ([bd6c468](https://github.com/simplyRoba/pixoo-bridge.rs/commit/bd6c468fa395cfc7593a70cbafa317c8a7b693a4))
* pixoo health check ([#34](https://github.com/simplyRoba/pixoo-bridge.rs/issues/34)) ([30f9c44](https://github.com/simplyRoba/pixoo-bridge.rs/commit/30f9c4478f40e9c12a0b6bb83145532282f2d10c))
* **pixoo:** add HTTP client with command payloads and response validation ([f09588b](https://github.com/simplyRoba/pixoo-bridge.rs/commit/f09588bc3431c02bb283b47d254882e04f0ba514))


### Bug Fixes

* **ci:** bump actions/cache from 4 to 5 ([#24](https://github.com/simplyRoba/pixoo-bridge.rs/issues/24)) ([7e8e424](https://github.com/simplyRoba/pixoo-bridge.rs/commit/7e8e424e926ca1b0f430dcdba2e21e3fa08e25be))
* **ci:** bump actions/checkout from 4 to 6 ([#28](https://github.com/simplyRoba/pixoo-bridge.rs/issues/28)) ([b1441da](https://github.com/simplyRoba/pixoo-bridge.rs/commit/b1441da54e0f118c8211002955a0b370e7dcd88f))
* **ci:** bump actions/upload-artifact from 4 to 6 ([#27](https://github.com/simplyRoba/pixoo-bridge.rs/issues/27)) ([e4b9aa7](https://github.com/simplyRoba/pixoo-bridge.rs/commit/e4b9aa7b0403e6e5e59c8889c52b9fa81d901d75))
* **ci:** bump docker/build-push-action from 5 to 6 ([#25](https://github.com/simplyRoba/pixoo-bridge.rs/issues/25)) ([1aeb97a](https://github.com/simplyRoba/pixoo-bridge.rs/commit/1aeb97aa3d1e01fe45c49e185a099d4df34e3dd2))
* **deps:** bump axum from 0.7.9 to 0.8.8 ([#26](https://github.com/simplyRoba/pixoo-bridge.rs/issues/26)) ([b68a10d](https://github.com/simplyRoba/pixoo-bridge.rs/commit/b68a10d52aea09a4ccf83b0072c0c85696a1d04a))
* **deps:** bump httpmock from 0.7.0 to 0.8.2 ([5c7f858](https://github.com/simplyRoba/pixoo-bridge.rs/commit/5c7f858f26023de5627cf2c282ed6c21b891907b))
* **docs:** update review and task instructions to include `cargo clippy` ([ef5ecd5](https://github.com/simplyRoba/pixoo-bridge.rs/commit/ef5ecd5af0ecd2d9fafce9b5f3b6d52035e386e0))
* **spec:** simplify task rules for branch management and work chunking ([88e9b32](https://github.com/simplyRoba/pixoo-bridge.rs/commit/88e9b32eef9ef276c617f8ba558d04e0594b3007))
* **spec:** update Purpose section to define baseline bridge foundation ([1223a97](https://github.com/simplyRoba/pixoo-bridge.rs/commit/1223a97715571c6364f02ab367f562d84bad30ab))

## [0.2.4](https://github.com/simplyRoba/pixoo-bridge.rs/compare/v0.2.3...v0.2.4) (2026-01-25)


### Bug Fixes

* install amd64 target before build ([#19](https://github.com/simplyRoba/pixoo-bridge.rs/issues/19)) ([e96b628](https://github.com/simplyRoba/pixoo-bridge.rs/commit/e96b62885e834496c21a7f6d6be6c82e9f85aaa1))

## [0.2.3](https://github.com/simplyRoba/pixoo-bridge.rs/compare/v0.2.2...v0.2.3) (2026-01-25)


### Bug Fixes

* trigger release ([2ca9d1d](https://github.com/simplyRoba/pixoo-bridge.rs/commit/2ca9d1da82769b734aa188fccdeb41a6b519438c))

## [0.2.2](https://github.com/simplyRoba/pixoo-bridge.rs/compare/v0.2.1...v0.2.2) (2026-01-25)


### Bug Fixes

* install arm64 target in Docker build ([#15](https://github.com/simplyRoba/pixoo-bridge.rs/issues/15)) ([f2c4163](https://github.com/simplyRoba/pixoo-bridge.rs/commit/f2c4163a0f1cbf162de729c761baebfc9f161c62))

## [0.2.1](https://github.com/simplyRoba/pixoo-bridge.rs/compare/v0.2.0...v0.2.1) (2026-01-25)


### Bug Fixes

* use separate builder stages for multi-arch support ([#13](https://github.com/simplyRoba/pixoo-bridge.rs/issues/13)) ([37a32ad](https://github.com/simplyRoba/pixoo-bridge.rs/commit/37a32ad9e98ed3a2e2a1b8a2d2e5729d25eef6c6))

## [0.2.0](https://github.com/simplyRoba/pixoo-bridge.rs/compare/v0.1.2...v0.2.0) (2026-01-25)


### Features

* update Rust to latest stable 1.92 ([cfbbe3a](https://github.com/simplyRoba/pixoo-bridge.rs/commit/cfbbe3a8bb3d279778125efd3fd149767cdb07b2))

## [0.1.2](https://github.com/simplyRoba/pixoo-bridge.rs/compare/v0.1.1...v0.1.2) (2026-01-25)


### Bug Fixes

* resolve Docker build COPY instruction error ([04cf9d7](https://github.com/simplyRoba/pixoo-bridge.rs/commit/04cf9d7877963a4cfb0603b80b7e18e4f07a9a98))

## [0.1.1](https://github.com/simplyRoba/pixoo-bridge.rs/compare/v0.1.0...v0.1.1) (2026-01-25)


### Bug Fixes

* use RELEASE_PLEASE_TOKEN to trigger publish workflow ([9e71b92](https://github.com/simplyRoba/pixoo-bridge.rs/commit/9e71b929631a063d2706e4c9e6c8a4dfb2af3bd9))

## 0.1.0 (2026-01-25)


### Features

* add ARM64 multi-platform support ([1834bff](https://github.com/simplyRoba/pixoo-bridge.rs/commit/1834bff290af5a44bfe175a8bc6f0194622d85eb))
* add change proposal for Rust bridge foundation and CI flow ([d59b009](https://github.com/simplyRoba/pixoo-bridge.rs/commit/d59b0097e8e800e82b3e740b84e142d7e483ca19))
* implement Rust bridge foundation and CI workflows ([a25a2c4](https://github.com/simplyRoba/pixoo-bridge.rs/commit/a25a2c4d88a50ccfe26ef5bc28a4d64dbc7249e2))


### Bug Fixes

* configure release-please to use PAT token ([347eab9](https://github.com/simplyRoba/pixoo-bridge.rs/commit/347eab9cf6ffe75a78cb8da82e26d85931dee453))
* remove dead code to pass clippy ([59d944a](https://github.com/simplyRoba/pixoo-bridge.rs/commit/59d944a80e1382d3049d893dfb9b6036c47b9aa9))
* rename artifacts to pixoo-bridge-rs to avoid conflicts ([dac5fbb](https://github.com/simplyRoba/pixoo-bridge.rs/commit/dac5fbbd79b6538ff17764d07ef4a826c9550441))
* rename token to RELEASE_PLEASE_TOKEN and remove setup docs ([58bbaff](https://github.com/simplyRoba/pixoo-bridge.rs/commit/58bbaff79fdfa89d224790f56915372505475793))
* use proper linker for ARM64 cross-compilation ([5e057df](https://github.com/simplyRoba/pixoo-bridge.rs/commit/5e057df3f8a111cccd4892808db021f40c1be73c))
