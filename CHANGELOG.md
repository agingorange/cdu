# Changelog

All notable changes to this project will be documented in this file.

### Format

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Entries should have the imperative form, just like commit messages. Start each entry with words
like add, fix, increase, force etc.. Not added, fixed, increased, forced etc.

### Categories each change fall into

* **Added**: for new features.
* **Changed**: for changes in existing functionality.
* **Deprecated**: for soon-to-be removed features.
* **Removed**: for now removed features.
* **Fixed**: for any bug fixes.
* **Security**: in case of vulnerabilities.

## [Unreleased]

## [0.1.4] - 2024-06-12

### Added

- Send a message via a Discord webhook if the outside IP changes.
- Use Dotenvy to load environment variables from a file.

### Changed

- Use `serde_jason::Value` instead of Cloudlflare's exact structs to prevent having to update the structs if something unrelated changes.
- Explain how to use Tracing in README.
- Update crates.

## [0.1.3] - 2024-05-16

### Added

- Add workflow to verify if new version number has been added to the change log and `Cargo.toml`.
- Add `--config-dir` flag to specify the directory for the configuration file.

### Fixed

- Fix incorrect environment variable names in README.

### Removed

- Remove unused crates.

## [0.1.2] - 2024-04-09

### Changed

- Remove unused targets from Release workflow.

### Security

- Bump dependencies because of h2 vulnerability.

## [0.1.1] - 2024-03-20

### Added

- Use cross for cross-compilation in the Release workflow.
- Add CHANGELOG and LICENSE to releases.
- Add MIT License.

### Changed

- Bump crates.

## [0.1.0] - 2024-03-16

### Added

- Initial release.
