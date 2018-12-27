use wasmi::{
    Error as InterpreterError, Trap, TrapKind, ModuleImportResolver, Externals,
    FuncRef, FuncInstance, Signature, ValueType, RuntimeValue, RuntimeArgs,
    nan_preserving_float::{F32, F64}
};

use super::state::{ContractState, StateValue};

struct CachedContractState {
    state: Vec<StateValue>
}

impl From<ContractState> for CachedContractState {
    fn from(contract: ContractState) -> Self {
        CachedContractState {
            state: contract.into_state()
        }
    }
}

const GET_INT32_INDEX: usize = 0;
const GET_INT64_INDEX: usize = 1;
const GET_FLOAT32_INDEX: usize = 2;
const GET_FLOAT64_INDEX: usize = 3;
const GET_MAPPING_INDEX: usize = 4;

const SET_INT32_INDEX: usize = 5;
const SET_INT64_INDEX: usize = 6;
const SET_FLOAT32_INDEX: usize = 7;
const SET_FLOAT64_INDEX: usize = 8;
const SET_MAPPING_INDEX: usize = 9;

impl Externals for CachedContractState {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            GET_INT32_INDEX => {
                let index: u32 = args.nth(0);
                match self.state.get(index as usize) {
                    Some(StateValue::U32(val)) => Ok(Some(RuntimeValue::I32(*val as i32))),
                    Some(_) => Err(Trap::new(TrapKind::Unreachable)),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds)),
                }
            },
            GET_INT64_INDEX => {
                let index: u32 = args.nth(0);
                match self.state.get(index as usize) {
                    Some(StateValue::U64(val)) => Ok(Some(RuntimeValue::I64(*val as i64))),
                    Some(_) => Err(Trap::new(TrapKind::Unreachable)),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds)),
                }
            },
            GET_FLOAT32_INDEX => {
                let index: u32 = args.nth(0);
                match self.state.get(index as usize) {
                    Some(StateValue::F32(val)) => Ok(Some(RuntimeValue::F32(F32::from(*val)))),
                    Some(_) => Err(Trap::new(TrapKind::Unreachable)),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds)),
                }
            },
            GET_FLOAT64_INDEX => {
                let index: u32 = args.nth(0);
                match self.state.get(index as usize) {
                    Some(StateValue::F64(val)) => Ok(Some(RuntimeValue::F64(F64::from(*val)))),
                    Some(_) => Err(Trap::new(TrapKind::Unreachable)),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds)),
                }
            },
            GET_MAPPING_INDEX => {
                let index: u32 = args.nth(0);
                let key: u64 = args.nth(1);
                match self.state.get(index as usize) {
                    Some(StateValue::Mapping(val)) => {
                        val.get(&key).ok_or_else(|| {
                            Trap::new(TrapKind::MemoryAccessOutOfBounds)
                        }).map(|val: &u64| { Some(RuntimeValue::I64(*val as i64)) })
                    },
                    Some(_) => Err(Trap::new(TrapKind::Unreachable)),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds)),
                }
            },


            SET_INT32_INDEX => {
                let index: u32 = args.nth(0);
                let value: u32 = args.nth(1);
                match self.state.get_mut(index as usize) {
                    Some(StateValue::U32(ref mut val)) => {
                        *val = value;
                        Ok(None)
                    },
                    Some(_) => Err(Trap::new(TrapKind::Unreachable)),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds)),
                }
            },
            SET_INT64_INDEX => {
                let index: u32 = args.nth(0);
                let value: u64 = args.nth(1);
                match self.state.get_mut(index as usize) {
                    Some(StateValue::U64(ref mut val)) => {
                        *val = value;
                        Ok(None)
                    },
                    Some(_) => Err(Trap::new(TrapKind::Unreachable)),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds)),
                }
            },
            SET_FLOAT32_INDEX => {
                let index: u32 = args.nth(0);
                let value: F32 = args.nth(1);
                match self.state.get_mut(index as usize) {
                    Some(StateValue::F32(ref mut val)) => {
                        *val = value.to_float();
                        Ok(None)
                    },
                    Some(_) => Err(Trap::new(TrapKind::Unreachable)),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds)),
                }
            },
            SET_FLOAT64_INDEX => {
                let index: u32 = args.nth(0);
                let value: F64 = args.nth(1);
                match self.state.get_mut(index as usize) {
                    Some(StateValue::F64(ref mut val)) => {
                        *val = value.to_float();
                        Ok(None)
                    },
                    Some(_) => Err(Trap::new(TrapKind::Unreachable)),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds)),
                }
            },
            SET_MAPPING_INDEX => {
                let index: u32 = args.nth(0);
                let key: u64 = args.nth(1);
                let value: u64 = args.nth(2);
                match self.state.get_mut(index as usize) {
                    Some(StateValue::Mapping(ref mut val)) => {
                        val.insert(key, value);
                        Ok(None)
                    },
                    Some(_) => Err(Trap::new(TrapKind::Unreachable)),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds)),
                }
            },

            _ => Err(Trap::new(TrapKind::Unreachable)),
        }
    }
}

pub struct Resolver;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Read;
    use std::collections::HashMap;

    use wasmi::{Module, ModuleInstance, ModuleRef, ImportsBuilder};

    use dag::contract::state::ContractState;

    fn load_module_from_file(filename: String) -> Module {
        let mut file = File::open(filename).expect("Could not open test file");
        let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf).expect("Could not read test file");
        Module::from_buffer(&buf).expect("Could not parse file into WASM module")
    }

    fn load_api_test_module_instance() -> ModuleRef {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/contracts/raw_api_test.wasm");
        let module = load_module_from_file(d.to_str().unwrap().to_string());

        let mut imports = ImportsBuilder::new();
        imports.push_resolver("env", &Resolver);
        ModuleInstance::new(&module, &imports)
            .expect("Failed to instantiate module")
            .assert_no_start()
    }

    #[test]
    fn test_api_resolver_u32() {
        let state = ContractState::new(vec![StateValue::U32(1)]);
        let mut temp_state = CachedContractState::from(state);
        let main = load_api_test_module_instance();

        assert_eq!(Some(RuntimeValue::I32(1)), main.invoke_export(
            "get_u32",
            &[RuntimeValue::I32(0)],
            &mut temp_state).unwrap());

        assert!(main.invoke_export(
            "set_u32",
            &[RuntimeValue::I32(0), RuntimeValue::I32(10)],
            &mut temp_state).is_ok());

        assert_eq!(Some(RuntimeValue::I32(10)), main.invoke_export(
            "get_u32",
            &[RuntimeValue::I32(0)],
            &mut temp_state).unwrap());

        // Error, because we are trying to retrieve a u64 where there is a u32
        assert!(main.invoke_export(
            "get_u64",
            &[RuntimeValue::I32(0)],
            &mut temp_state).is_err());

        // Error, incorrect arguments
        assert!(main.invoke_export(
            "get_u32",
            &[RuntimeValue::I64(0)],
            &mut temp_state).is_err());
    }

    #[test]
    fn test_api_resolver_u64() {
        let state = ContractState::new(vec![StateValue::U64(1)]);
        let mut temp_state = CachedContractState::from(state);
        let main = load_api_test_module_instance();

        assert_eq!(Some(RuntimeValue::I64(1)), main.invoke_export(
            "get_u64",
            &[RuntimeValue::I32(0)],
            &mut temp_state).unwrap());

        assert!(main.invoke_export(
            "set_u64",
            &[RuntimeValue::I32(0), RuntimeValue::I64(10)],
            &mut temp_state).is_ok());

        assert_eq!(Some(RuntimeValue::I64(10)), main.invoke_export(
            "get_u64",
            &[RuntimeValue::I32(0)],
            &mut temp_state).unwrap());

        // Error, because we are trying to retrieve a u32 where there is a u64
        assert!(main.invoke_export(
            "get_u32",
            &[RuntimeValue::I32(0)],
            &mut temp_state).is_err());
    }

    #[test]
    fn test_api_resolver_f32() {
        let state = ContractState::new(vec![StateValue::F32(1f32)]);
        let mut temp_state = CachedContractState::from(state);
        let main = load_api_test_module_instance();

        assert_eq!(Some(RuntimeValue::F32(1f32.into())), main.invoke_export(
            "get_f32",
            &[RuntimeValue::I32(0)],
            &mut temp_state).unwrap());

        assert!(main.invoke_export(
            "set_f32",
            &[RuntimeValue::I32(0), RuntimeValue::F32(10f32.into())],
            &mut temp_state).is_ok());

        assert_eq!(Some(RuntimeValue::F32(10f32.into())), main.invoke_export(
            "get_f32",
            &[RuntimeValue::I32(0)],
            &mut temp_state).unwrap());

        // Error, because we are trying to retrieve a f64 where there is a f32
        assert!(main.invoke_export(
            "get_f64",
            &[RuntimeValue::I32(0)],
            &mut temp_state).is_err());
    }

    #[test]
    fn test_api_resolver_f64() {
        let state = ContractState::new(vec![StateValue::F64(1f64)]);
        let mut temp_state = CachedContractState::from(state);
        let main = load_api_test_module_instance();

        assert_eq!(Some(RuntimeValue::F64(1f64.into())), main.invoke_export(
            "get_f64",
            &[RuntimeValue::I32(0)],
            &mut temp_state).unwrap());

        assert!(main.invoke_export(
            "set_f64",
            &[RuntimeValue::I32(0), RuntimeValue::F64(10f64.into())],
            &mut temp_state).is_ok());

        assert_eq!(Some(RuntimeValue::F64(10f64.into())), main.invoke_export(
            "get_f64",
            &[RuntimeValue::I32(0)],
            &mut temp_state).unwrap());

        // Error, because we are trying to retrieve a f64 where there is a f32
        assert!(main.invoke_export(
            "get_f32",
            &[RuntimeValue::I32(0)],
            &mut temp_state).is_err());
    }

    #[test]
    fn test_api_resolver_mapping() {
        let state = ContractState::new(vec![StateValue::Mapping(HashMap::new())]);
        let mut temp_state = CachedContractState::from(state);
        let main = load_api_test_module_instance();

        assert!(main.invoke_export(
            "get_mapping",
            &[RuntimeValue::I32(0), RuntimeValue::I64(0)],
            &mut temp_state).is_err());

        assert!(main.invoke_export(
            "set_mapping",
            &[RuntimeValue::I32(0), RuntimeValue::I64(0), RuntimeValue::I64(0)],
            &mut temp_state).is_ok());
        assert!(main.invoke_export(
            "set_mapping",
            &[RuntimeValue::I32(0), RuntimeValue::I64(1), RuntimeValue::I64(10)],
            &mut temp_state).is_ok());

        assert_eq!(Some(RuntimeValue::I64(0)), main.invoke_export(
            "get_mapping",
            &[RuntimeValue::I32(0), RuntimeValue::I64(0)],
            &mut temp_state).unwrap());
        assert_eq!(Some(RuntimeValue::I64(10)), main.invoke_export(
            "get_mapping",
            &[RuntimeValue::I32(0), RuntimeValue::I64(1)],
            &mut temp_state).unwrap());
    }
}