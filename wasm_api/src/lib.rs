macro_rules! externs {
    ($(fn $name:ident($($args:ident: $args_type:ty),*) -> $ret:ty;)*) => (
        #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
        mod sys {
            extern "C" {
                $(pub fn $name($($args: $args_type,)*) -> $ret;)*
            }
        }

        $(
            #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
            pub fn $name($($args: $args_type,)*) -> $ret {
                unsafe { sys::$name($($args,)*) }
            }
        )*

        $(
            #[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
            #[allow(unused_variables)]
            pub fn $name($($args: $args_type,)*) -> $ret {
                panic!("function not implemented on non-wasm32 targets")
            }
        )*
    )
}

externs! {
    fn api_get_u32(index: u32) -> u32;
    fn api_get_u64(index: u32) -> u64;
    fn api_get_f32(index: u32) -> f32;
    fn api_get_f64(index: u32) -> f64;
    fn api_get_mapping(index: u32, key: u64) -> u64;

    fn api_set_u32(index: u32, value: u32) -> ();
    fn api_set_u64(index: u32, value: u64) -> ();
    fn api_set_f32(index: u32, value: f32) -> ();
    fn api_set_f64(index: u32, value: f64) -> ();
    fn api_set_mapping(index: u32, key: u64, value: u64) -> ();
}
