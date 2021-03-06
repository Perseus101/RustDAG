mod api {
    mod sys {
        extern {
            pub fn api_get_u32(index: u32) -> u32;
            pub fn api_get_u64(index: u32) -> u64;
            pub fn api_get_f32(index: u32) -> f32;
            pub fn api_get_f64(index: u32) -> f64;
            pub fn api_get_mapping(index: u32, key: u64) -> u64;

            pub fn api_set_u32(index: u32, value: u32);
            pub fn api_set_u64(index: u32, value: u64);
            pub fn api_set_f32(index: u32, value: f32);
            pub fn api_set_f64(index: u32, value: f64);
            pub fn api_set_mapping(index: u32, key: u64, value: u64);
        }
    }

    pub fn get_u32(index: u32) -> u32 {
        unsafe { sys::api_get_u32(index) }
    }
    pub fn get_u64(index: u32) -> u64 {
        unsafe { sys::api_get_u64(index) }
    }
    pub fn get_f32(index: u32) -> f32 {
        unsafe { sys::api_get_f32(index) }
    }
    pub fn get_f64(index: u32) -> f64 {
        unsafe { sys::api_get_f64(index) }
    }
    pub fn get_mapping(index: u32, key: u64) -> u64 {
        unsafe { sys::api_get_mapping(index, key) }
    }

    pub fn set_u32(index: u32, value: u32) {
        unsafe { sys::api_set_u32(index, value) }
    }
    pub fn set_u64(index: u32, value: u64) {
        unsafe { sys::api_set_u64(index, value) }
    }
    pub fn set_f32(index: u32, value: f32) {
        unsafe { sys::api_set_f32(index, value) }
    }
    pub fn set_f64(index: u32, value: f64) {
        unsafe { sys::api_set_f64(index, value) }
    }
    pub fn set_mapping(index: u32, key: u64, value: u64) {
        unsafe { sys::api_set_mapping(index, key, value) }
    }
}

#[no_mangle]
pub fn init() {
    api::set_u32(0, 1);
    api::set_u64(1, 2);
    api::set_f32(2, 3f32);
    api::set_f64(3, 4f64);
    api::set_mapping(4, 0, 5);
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
