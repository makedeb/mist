# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.4] - 2022-09-25
### Fixed
- Ensure cache directories exist before they're used.

## [0.9.3] - 2022-09-25
### Fixed
- Add `postinst` script during deployments.

## [0.9.2] - 2022-09-25
### Fixed
- Add `postinst` script to set permissions on executable in installed package.
- Ensure MPR cache files exist before trying to write to them.

## [0.9.1] - 2022-09-25
### Fixed
- Allow passing the `NO_SUDO` environment variable to builds.

## [0.9.0] - 2022-09-25
### Added
- Added `install` command.
- Added `upgrade` command.
- Added `remove` command.
- Added `list` command.

### Removed
- Removed `info` command.

## [0.8.0] - 2022-08-04
### Changed
- Renamed project back to `Mist`.

## [0.7.0] - 2022-08-04
### Changed
- Renamed project to `Mari`.

## [0.6.2] - 2022-08-03
### Added
- Added symlink in PKGBUILD for ease of transition from `mpr-cli` to `mist`.

## [0.6.1] - 2022-08-03
### Added
- Added needed fields in PKGBUILD for transition from `mpr-cli` to `mist`.

## [0.6.0] - 2022-08-03
### Changed
- Renamed project to `Mist`.

## [0.5.0] - 2022-07-23
### Added
- Added `update` command.

## [0.4.2] - 2022-07-11
Internal changes used to test CI. No changes have been made for end users.

## [0.4.1] - 2022-07-11
Internal changes used to test CI. No changes have been made for end users.

## [0.4.0] - 2022-07-11
### Added
- Added APT integration to `search` and `info` commands.

## [0.3.4] 2022-07-11
### Changed
Internal changes used to test CI. No changes have been made for end users.

## [0.3.3] - 2022-07-02
### Security
- Updated dependencies.

## [0.3.2] - 2022-06-25
### Fixed
- Make `libssl-dev` a runtime dependency instead of just a build dependency.

## [0.3.1] - 2022-06-14
### Fixed
- Fixed `sed` command used to set version in man page during builds.

## [0.3.0] - 2022-06-13
### Added
- Add `info` command.
- Add `comment` command.
- Add `list-comments` command.

## [0.2.0] - 2022-06-11
### Added
- Add `clone` command.

## [0.1.1] - 2022-06-06
### Fixed
- Recursively created cache directory if it doesn't exist.

## [0.1.0] - 2022-06-06
The beginning of the project! 🥳

### Added
- The beginning of the project!
- Add `search` command.
- Add `whoami` command.
