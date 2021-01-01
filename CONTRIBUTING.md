# Contributing

## Expectations

There are only a handful of expectations when contributing to this project:

1. All new features should have associated tests
2. All new features should have associated documentation either in the `docs/`
   directory or as a Rustdoc comment
3. All changes to existing features should have updated documentation & tests
4. All contributions should be reflected in the `CHANGELOG.md` file so we can
   properly track changes between versions

## Development

This project maintains three supported versions:

- Rust 1.45.0 (minimum supported version of Rust)
- Stable (latest stable version of Rust)
- Nightly (latest nightly version of Rust)

The following instructions describe building, testing, and formatting code
based on these versions.

### Building

Building this project is straightforward using `cargo build` from the root of
the project.

### Testing

For most test work, you can run `cargo test` from the root of the project
to cover the everyday logic. The CI Github Action job will run tests against
1.45.0 and nightly to ensure that nothing has been broken.

The UI tests in **entity_macros** will only run on nightly, meaning that
from the **entity_macros** project you need to run `cargo +nightly test`
in order to trigger the UI tests.

For **entity**, there are several features that are not enabled by
default (with associated tests not running). The CI job will also run tests
to verify compilation without any default features and with all
features to make sure we have coverage across different variations of the
crate.

### Formatting

All Rust files must following `rustfmt`. It is expected that any pull request
is properly formatted prior to being submitted. The CI Github Action will
catch if one or more files are not formatted.

To format, run `cargo fmt --all` from the root of the project.
