mod api {
    mod sys {
        extern {
            pub fn __ofc__get_u32(index: u32) -> u32;
            pub fn __ofc__get_u64(index: u32) -> u64;
            pub fn __ofc__get_f32(index: u32) -> f32;
            pub fn __ofc__get_f64(index: u32) -> f64;
            pub fn __ofc__get_mapping(index: u32, key: u64) -> u64;

            pub fn __ofc__set_u32(index: u32, value: u32);
            pub fn __ofc__set_u64(index: u32, value: u64);
            pub fn __ofc__set_f32(index: u32, value: f32);
            pub fn __ofc__set_f64(index: u32, value: f64);
            pub fn __ofc__set_mapping(index: u32, key: u64, value: u64);
        }
    }

    pub fn get_u32(index: u32) -> u32 {
        unsafe { sys::__ofc__get_u32(index) }
    }
    pub fn get_u64(index: u32) -> u64 {
        unsafe { sys::__ofc__get_u64(index) }
    }
    pub fn get_f32(index: u32) -> f32 {
        unsafe { sys::__ofc__get_f32(index) }
    }
    pub fn get_f64(index: u32) -> f64 {
        unsafe { sys::__ofc__get_f64(index) }
    }
    pub fn get_mapping(index: u32, key: u64) -> u64 {
        unsafe { sys::__ofc__get_mapping(index, key) }
    }

    pub fn set_u32(index: u32, value: u32) {
        unsafe { sys::__ofc__set_u32(index, value) }
    }
    pub fn set_u64(index: u32, value: u64) {
        unsafe { sys::__ofc__set_u64(index, value) }
    }
    pub fn set_f32(index: u32, value: f32) {
        unsafe { sys::__ofc__set_f32(index, value) }
    }
    pub fn set_f64(index: u32, value: f64) {
        unsafe { sys::__ofc__set_f64(index, value) }
    }
    pub fn set_mapping(index: u32, key: u64, value: u64) {
        unsafe { sys::__ofc__set_mapping(index, key, value) }
    }
}

/////////////////////// Initial state size markers ///////////////////////
#[no_mangle]
#[allow(non_snake_case)]
pub fn __ofc__state_u32() -> u32 {
    1
}
#[no_mangle]
#[allow(non_snake_case)]
pub fn __ofc__state_u64() -> u32 {
    1
}
#[no_mangle]
#[allow(non_snake_case)]
pub fn __ofc__state_f32() -> u32 {
    1
}
#[no_mangle]
#[allow(non_snake_case)]
pub fn __ofc__state_f64() -> u32 {
    1
}
#[no_mangle]
#[allow(non_snake_case)]
pub fn __ofc__state_mapping() -> u32 {
    1
}

#[no_mangle]
pub fn init() {
    api::set_u32(0, 1);
    api::set_u64(0, 1);
    api::set_f32(0, 1f32);
    api::set_f64(0, 1f64);
    api::set_mapping(0, 0, 1);
}
/////////////////////// Contract functions ///////////////////////
#[no_mangle]
pub fn get_u32(index: u32) -> u32 {
    api::get_u32(index)
}
#[no_mangle]
pub fn get_u64(index: u32) -> u64 {
    api::get_u64(index)
}
#[no_mangle]
pub fn get_f32(index: u32) -> f32 {
    api::get_f32(index)
}
#[no_mangle]
pub fn get_f64(index: u32) -> f64 {
    api::get_f64(index)
}
#[no_mangle]
pub fn get_mapping(index: u32, key: u64) -> u64 {
    api::get_mapping(index, key)
}

#[no_mangle]
pub fn set_u32(index: u32, value: u32) {
    api::set_u32(index, value)
}
#[no_mangle]
pub fn set_u64(index: u32, value: u64) {
    api::set_u64(index, value)
}
#[no_mangle]
pub fn set_f32(index: u32, value: f32) {
    api::set_f32(index, value)
}
#[no_mangle]
pub fn set_f64(index: u32, value: f64) {
    api::set_f64(index, value)
}
#[no_mangle]
pub fn set_mapping(index: u32, key: u64, value: u64) {
    api::set_mapping(index, key, value)
}