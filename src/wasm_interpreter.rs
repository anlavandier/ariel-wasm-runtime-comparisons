use wasm::{validate, RuntimeInstance, Value};

extern crate alloc;
use alloc::vec::Vec;

fn extra(_: &mut (), _: Vec<Value>) -> Vec<Value> {
    Vec::from_iter(core::iter::once(Value::I32(100)))
}

pub fn run_wasm(a: u32, b:u32, _c: u32) -> u32{
    let wasm_bytes = include_bytes!("../input.wasm");

    let validation_info = validate(wasm_bytes).unwrap();

    let mut instance = RuntimeInstance::new(());

    instance.add_host_function_typed::<(), u32>("host", "extra", extra).unwrap();

    instance.add_module("module", &validation_info).unwrap();

    let res: u32 = instance.invoke_typed(
        &instance.get_function_by_name("module", "add_with_extra").unwrap(),
        (a, b)).unwrap();

    return res
}