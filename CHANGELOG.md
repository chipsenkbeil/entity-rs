# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Changed

- Refactored `entity` and `entity_macros` into the following crates:
    - `entity`: contains the core traits and data structures
    - `entity_macros`: contains the macros used to generate `entity` traits
    - `entity_macros_data`: contains common `darling` data structures to parse
      the `entity` structs and enums for use in macro creation
    - `entity_noop_macros`: provides shim macros when certain features are
      excluded
    - `entity_async_graphql`: contains the core traits and data structures to
      interface with `async-graphql`
    - `entity_async_graphql_derive`: contains the macros used to generate the
      async-graphql wrappers for ents
    - `entity_inmemory`: contains a database implementation for inmemory usage
    - `entity_sled`: contains a database implementation using `sled`
- `EntQuery` trait now has `execute` method use the global database and
  exposes `execute_with_db` to take in a `WeakDatabaseRc` rather than a
  generic database in order to be more consistent with other APIs

### Added

- This `CHANGELOG.md` file to keep track of future changes
- A `release.toml` using `cargo-release` to keep track of all version changes
  and update multiple `Cargo.toml` as well as other files like this changelog

### Fixed

- Addressed hygiene issues with the macros associated with `async-graphql`
