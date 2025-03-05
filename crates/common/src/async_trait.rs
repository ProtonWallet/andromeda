#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! async_trait_impl {
    ($($impl:tt)*) => {
        #[async_trait::async_trait(?Send)]
        $($impl)*
    };
}
/// Notes: macro for wasm32 check, orignal looks like this
/// #[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
/// #[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! async_trait_impl {
    ($($impl:tt)*) => {
        #[async_trait::async_trait]
        $($impl)*
    };
}
