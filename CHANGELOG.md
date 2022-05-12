# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0](https://www.github.com/arcana-engine/sierra/compare/v0.4.0...v0.5.0) (2022-05-12)


### Features

* Add release-please action ([5e7aa82](https://www.github.com/arcana-engine/sierra/commit/5e7aa821796db84e55e954d2d8fc2f6b6b2f3173))
* Support swizzling ([de8a739](https://www.github.com/arcana-engine/sierra/commit/de8a739ef203bcf5cda6160ad88193e258d7cbe4))
* Swizzling in Vulkan ([4c5f4fb](https://www.github.com/arcana-engine/sierra/commit/4c5f4fb47cf94dc6157a4536edc47ae6a98b5e0e))


### Bug Fixes

* Better borrowing for descriptors ([d4842b3](https://www.github.com/arcana-engine/sierra/commit/d4842b318a8611690cb23188f514c410016b489d))
* cargo fmt run ([b72ee62](https://www.github.com/arcana-engine/sierra/commit/b72ee62af2c249de5f29c37e8df90ede8ffe2327))
* clippy ([7316a16](https://www.github.com/arcana-engine/sierra/commit/7316a1678fa5faac3b61cf4c6c44c0a6a803af39))
* Clippy ([c1627dd](https://www.github.com/arcana-engine/sierra/commit/c1627dd7728c800ad10b85c373cddd6c56c615de))
* s/main/master in release-please.yml ([#9](https://www.github.com/arcana-engine/sierra/issues/9)) ([cb665a6](https://www.github.com/arcana-engine/sierra/commit/cb665a61e02136e17ca4932bec022e8f0ecd3230))

## [Unreleased]

## [0.2.0] - 2021-06-29

### Added
- This changelog
- Resource usage tracking with epoch based reclamation.
- Support for descriptor arrays in `descriptors` proc macro

### Changed
- Swapchain image ergonomics
- Descriptor sets now allocated with gpu-descriptor
