# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Fixed bug introduced when removing brace escaping support that lead to out-of-bound panics when two bound parameter references were too far apart. ([#4](https://github.com/kyrias/sqlx-conditional-queries/issues/4))
