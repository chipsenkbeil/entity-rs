# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.4.0] - 2022-10-11
### Fixed
  #80 pattern `Binary(_)` not covered
### Added
  crate bytes
### Changed
  use async-graphql 4.0.0

## [0.3.2] - 2021-04-24

### Fixed

- `entity_macros` generated builder no longer treats computed fields as
  regular fields, meaning that it no longer expects computed fields to
  be provided as part of the builder

## [0.3.1] - 2021-04-24

### Added

- `clear_cache` method added to `Ent` trait
- `field_definition` and `edge_definition` methods with default
  implementations added to `Ent` trait
- `ent(field(computed = "..."))` added to `entity_macros` crate
- `#[gql_ent]` attribute macro for `entity-async-graphql-macros` to support
  adding basic `async-graphql` derives alongside `#[simple_ent]`

### Changed

- `entity-inmemory` and `entity-sled` will now call `clear_cache` on ents that
  are inserted prior to saving them
- `Ent::field(...)` will now return `EntMutationError::ImmutableField` for
  immutable fields on generated ents via `entity_macros`
- `Ent::field(...)` will now return `EntMutationError::ComputedField` for
  computed fields on generated ents via `entity_macros`
- `EntMutationError` now includes `ComputedField` variant
- `DatabaseError` now includes `EntMutationFailed` variant

### Fixed

- `f32` and `f64` types were missing from `entity-async-graphql` filter
  types and have now been added and are supported

## [0.3.0] - 2021-04-22

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
    - `entity_async_graphql_macros`: contains the macros used to generate the
      async-graphql wrappers for ents
    - `entity_inmemory`: contains a database implementation for inmemory usage
    - `entity_sled`: contains a database implementation using `sled`
- `EntQuery` trait now has `execute` method use the global database and
  exposes `execute_with_db` to take in a `WeakDatabaseRc` rather than a
  generic database in order to be more consistent with other APIs

### Added

- This `CHANGELOG.md` file to keep track of future changes
- `scripts/release.sh` to keep track of all version changes and update multiple
  `Cargo.toml` as well as other files like this changelog
- Dedicated `README.md` files for each of the different crates

### Fixed

- Addressed hygiene issues with the macros associated with `async-graphql`
