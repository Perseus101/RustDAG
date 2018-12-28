use wasmi::{
    Error as InterpreterError, ModuleImportResolver, ImportsBuilder,
    FuncRef, FuncInstance, Signature, ValueType
};

pub const GET_INT32_INDEX: usize = 0;
pub const GET_INT64_INDEX: usize = 1;
pub const GET_FLOAT32_INDEX: usize = 2;
pub const GET_FLOAT64_INDEX: usize = 3;
pub const GET_MAPPING_INDEX: usize = 4;

pub const SET_INT32_INDEX: usize = 5;
pub const SET_INT64_INDEX: usize = 6;
pub const SET_FLOAT32_INDEX: usize = 7;
pub const SET_FLOAT64_INDEX: usize = 8;
pub const SET_MAPPING_INDEX: usize = 9;

pub struct Resolver;

pub fn get_imports_builder<'a>() -> ImportsBuilder<'a> {
    let mut imports = ImportsBuilder::new();
    imports.push_resolver("env", &Resolver);
    imports
}

impl ModuleImportResolver for Resolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, InterpreterError> {
        let func_ref = match field_name {
            "__ofc__get_u32" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                GET_INT32_INDEX,
            ),
            "__ofc__get_u64" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I64)),
                GET_INT64_INDEX,
            ),
            "__ofc__get_f32" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::F32)),
                GET_FLOAT32_INDEX,
            ),
            "__ofc__get_f64" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::F64)),
                GET_FLOAT64_INDEX,
            ),
            "__ofc__get_mapping" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I64][..],
                Some(ValueType::I64)), GET_MAPPING_INDEX,
            ),

            "__ofc__set_u32" => FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I32][..], None
                ),
                SET_INT32_INDEX,
            ),
            "__ofc__set_u64" => FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I64][..], None
                ),
                SET_INT64_INDEX,
            ),
            "__ofc__set_f32" => FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::F32][..], None
                ),
                SET_FLOAT32_INDEX,
            ),
            "__ofc__set_f64" => FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::F64][..], None
                ),
                SET_FLOAT64_INDEX,
            ),
            "__ofc__set_mapping" => FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I64, ValueType::I64][..], None
                ),
                SET_MAPPING_INDEX,
            ),
            _ => {
                return Err(InterpreterError::Function(format!(
                    "host module doesn't export function with name {}",
                    field_name
                )));
            }
        };
        Ok(func_ref)
    }
}