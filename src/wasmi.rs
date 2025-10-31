use wasmi::{Caller, Config, Engine, Linker, Module, Store};

pub fn run_wasm(a: u32, b: u32, c: u32) -> u32 {

    let wasm = include_bytes!("../input.wasm");

    let mut config = Config::default();

    config.floats(false);

    let engine = Engine::new(&config);
    let mut store = Store::new(&engine, ());

    let module = unsafe { Module::new_unchecked(&engine, wasm).unwrap() };

    let mut linker = Linker::new(&engine);

    linker.func_wrap("host", "extra", move |_: Caller<'_, _>| { c }).unwrap();

    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();

    instance.get_typed_func::<(u32, u32), u32>(&mut store, "add_with_extra").unwrap()
        .call(&mut store, (a, b)).unwrap()
}