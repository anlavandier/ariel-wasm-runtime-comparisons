use wasmtime::{Caller, Config, Engine, Linker, Module, Store};

pub fn run_wasm(a: u32, b: u32, c: u32) -> u32 {
    let wasm_input = include_bytes!("../input.cwasm");


    let mut config = Config::new();

    // Options that must conform with the precompilation step
    config.target("pulley32").unwrap();

    config.table_lazy_init(false);
    config.memory_reservation(0);
    config.memory_init_cow(false);
    config.memory_may_move(false);

    // Options that can be changed without changing the payload
    config.max_wasm_stack(2048);
    config.memory_reservation_for_growth(0);


    let engine = Engine::new(&config).unwrap();

    let mut store = Store::new(&engine, ());

    // SAFETY: This is a known input produced by Engine::precompile_module
    // Also, deserialize_raw reuse the given memory instead of copying it.
    let module = unsafe { Module::deserialize_raw(&engine, wasm_input.as_slice().into()).unwrap() };

    let mut linker = Linker::new(&engine);

    // Define the imported host function
    linker.func_wrap("host", "extra", move |_: Caller<'_, _>| { c }).unwrap();

    // Instantiate the Module
    let instance = linker.instantiate(&mut store, &module).unwrap();

    // call add_with_extra
    instance.get_typed_func::<(u32, u32), u32>(&mut store, "add_with_extra").unwrap()
        .call(&mut store, (a, b)).unwrap()
}


// Same as https://github.com/bytecodealliance/wasmtime/blob/main/examples/min-platform/embedding/wasmtime-platform.c
// I have no idea whether this is safe or not.
// https://github.com/bytecodealliance/wasmtime/blob/aec935f2e746d71934c8a131be15bbbb4392138c/crates/wasmtime/src/runtime/vm/traphandlers.rs#L888
static mut TLS_PTR: u32 = 0;

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
extern "C" fn wasmtime_tls_get() -> *mut u8 {
    #[allow(unsafe_code)]
    unsafe { TLS_PTR as *mut u8 }
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
extern "C" fn wasmtime_tls_set(val: *const u8) {
    #[allow(unsafe_code)]
    unsafe { TLS_PTR = val as u32 };
}