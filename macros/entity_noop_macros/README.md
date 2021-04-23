# entity_noop_macros

Used only as no-op for macros in feature-dependent deps.

Can be removed once https://github.com/rust-lang/rust/issues/64797 is available on stable.

```rust
#[cfg(feature = "typetag")]
pub use typetag::serde as typetag_serde;

#[cfg(not(feature = "typetag"))]
pub use entity_noop_macros::noop_attr as typetag_serde;
```
