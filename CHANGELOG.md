# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.3.2] - 2025-02-03

### Fixed
- Catch duplicate compile-time bindings and cyclical compile-time binding references.
  ([#26](https://github.com/kyrias/sqlx-conditional-queries/pull/26))


## [0.3.1] - 2025-01-09

### Changed
- Switch from `proc-macro-error` to `proc-macro-error2`.
  ([#24](https://github.com/kyrias/sqlx-conditional-queries/pull/24))


## [0.3.0] - 2025-01-07

### Fixed
- Fix queries only containing run-time bound parameters.
  ([#20](https://github.com/kyrias/sqlx-conditional-queries/issues/20),
  [#22](https://github.com/kyrias/sqlx-conditional-queries/pull/22))

### Changed
- Upgrade all dependencies.
  ([#23](https://github.com/kyrias/sqlx-conditional-queries/pull/23))
- Made `sqlx-conditional-queries-core` take database type at run-time.
  ([#21](https://github.com/kyrias/sqlx-conditional-queries/pull/21))


## [0.2.1] - 2024-08-19

### Fixed
- Internal dependency versions weren't updated in 0.2.0.


## [0.2.0] - 2024-08-19

### Changed
- Upgrade all dependencies. ([#19](https://github.com/kyrias/sqlx-conditional-queries/pull/19))


## [0.1.4] - 2024-03-27

### Deprecated

- Deprecated `fetch_many` method since it was deprecated in `sqlx` 0.7.4. ([#17](https://github.com/kyrias/sqlx-conditional-queries/pull/17))


## [0.1.3] - 2023-07-12

### Changed

- Drop patch version bound of internal crates.
- Switch from using type ascription synatx to using `as` for type overrides. ([#12](https://github.com/kyrias/sqlx-conditional-queries/issues/12), [#13](https://github.com/kyrias/sqlx-conditional-queries/issues/13))


## [0.1.2] - 2023-02-16

### Fixed

- Fixed bug introduced when removing brace escaping support that lead to out-of-bound panics when two bound parameter references were too far apart. ([#4](https://github.com/kyrias/sqlx-conditional-queries/issues/4))


[Unreleased]: https://github.com/kyrias/sqlx-conditional-queries/compare/0.3.2...main
[0.3.1]: https://github.com/kyrias/sqlx-conditional-queries/compare/0.3.1...0.3.2
[0.3.1]: https://github.com/kyrias/sqlx-conditional-queries/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/kyrias/sqlx-conditional-queries/compare/0.2.1...0.3.0
[0.2.1]: https://github.com/kyrias/sqlx-conditional-queries/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/kyrias/sqlx-conditional-queries/compare/0.1.4...0.2.0
[0.1.3]: https://github.com/kyrias/sqlx-conditional-queries/compare/0.1.3...0.1.4
[0.1.3]: https://github.com/kyrias/sqlx-conditional-queries/compare/0.1.2...0.1.3
[0.1.2]: https://github.com/kyrias/sqlx-conditional-queries/compare/0.1.1...0.1.2
