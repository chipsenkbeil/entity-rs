/// General purpose noop for function macro
#[proc_macro]
pub fn noop_func(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}

/// General purpose noop for attribute macro
#[proc_macro_attribute]
pub fn noop_attr(
    _: proc_macro::TokenStream,
    body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    body
}

/// General purpose noop for derive macro
#[proc_macro_derive(NoopDerive)]
pub fn noop_derive(_items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}

/// Specialized noop for serde as it needs the serde attribute
#[proc_macro_derive(NoopDeriveSerde, attributes(serde))]
pub fn noop_derive_serde(_items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}

/// Specialized noop for async_graphql as it needs the graphql attribute
#[proc_macro_derive(NoopDeriveAsyncGraphql, attributes(graphql))]
pub fn noop_derive_async_graphql(_items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}
