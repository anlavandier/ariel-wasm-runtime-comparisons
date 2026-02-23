use wasm::{validate, Store, ExternVal, Value, HaltExecutionError};

extern crate alloc;
use alloc::{vec, vec::Vec};

#[cfg(feature = "coremark")]
pub mod coremark {
    use super::*;

    fn clock_ms(_: &mut (), _: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
        Ok(Vec::from_iter(core::iter::once(Value::I64(ariel_os::time::Instant::now().as_millis()))))
    }

    pub fn run_coremark() -> f32 {
        let wasm_bytes = include_bytes!(crate::benchmark_file!());

        let validation_info = validate(wasm_bytes).unwrap();

        let mut store = Store::new(());

        let func_addr = store.func_alloc_typed::<(), u64>(clock_ms);

        let module = store.module_instantiate(
            &validation_info,
            vec![ExternVal::Func(func_addr)],
            None,
        ).unwrap();

        let run_addr = store.instance_export(module.module_addr, "run").unwrap()
            .as_func()
            .unwrap();

        let res: f32 = store.invoke_typed_without_fuel(
            run_addr, ()
        ).unwrap();
        return res
    }
}

#[cfg(feature = "embench-1")]
pub mod embench1 {
    use ariel_os::time::Instant;
    use libm::{pow, exp, log, sqrt};
    use ariel_os::debug::log::{debug, error};

    use super::*;
    use crate::{BENCH_SCORE, BENCHMARK_LOOPS, benchmark_name, benchmark_file};

    struct TimeTracking(Instant, Instant);

    use wasm::config::Config;

    impl Config for TimeTracking {}

    pub fn run_bench() -> (f64, f64, f64, f64) {
        let bench_name = benchmark_name!();
        let wasm = include_bytes!(benchmark_file!());

        let validation_info = validate(wasm).unwrap();

        let mut store = Store::new(TimeTracking(Instant::now(), Instant::now()));

        let board_init = store.func_alloc_typed::<(), ()>(initialise_board);
        let start_trig = store.func_alloc_typed::<(), ()>(start_trigger);
        let stop_trig = store.func_alloc_typed::<(), ()>(stop_trigger);

        let module = store.module_instantiate(
            &validation_info,
            vec![ExternVal::Func(board_init), ExternVal::Func(start_trig), ExternVal::Func(stop_trig)],
            None,
        ).unwrap();

        let bench_function = store.instance_export(module.module_addr, "__original_main").unwrap()
            .as_func()
            .unwrap();

        debug!("Starting wasm app");
        let mut times_to_run = Vec::new();
        for i in 1..=BENCHMARK_LOOPS {
            debug!("Run {}", i);
            let correct = store.invoke_typed_without_fuel(bench_function, ()).unwrap();
            match correct {
                0 => {
                    let &TimeTracking(start, end) = &store.user_data;
                    // The relative speed is in milli seconds
                    times_to_run.push((end - start).as_millis());
                },
                _ => {
                    error!("Benchmarking went wrong for some reason, aborting");
                    return (0_f64, 0_f64, 0_f64, 0_f64);
                }
            }
        }
        let mut geo_mean = 1_f64;
        let mut times_geo_mean = 1_f64;
        let score_to_div = BENCH_SCORE.iter().find(|(b_name, _)| *b_name == bench_name).unwrap().1;

        for dur in times_to_run.iter() {
            let normalized_speed = score_to_div as f64 / *dur as f64;
            geo_mean *= pow(normalized_speed as f64, 1_f64/BENCHMARK_LOOPS as f64);
            times_geo_mean *= pow(*dur as f64, 1_f64/BENCHMARK_LOOPS as f64);
        }

        // sigma = exp( sqrt( 1/N sum( ln ( A_i / mean )^2 ) ) ) https://en.wikipedia.org/wiki/Geometric_standard_deviation
        let mut times_geo_std = 0_f64;
        let mut geo_std = 0_f64;
        for dur in times_to_run.iter() {
            let normalized_speed = score_to_div as f64 / *dur as f64;
            let logged = log(normalized_speed / geo_mean);
            geo_std += logged * logged;

            let logged_times = log(*dur as f64 / times_geo_mean);
            times_geo_std += logged_times * logged_times;
        }
        geo_std = exp(sqrt(1_f64 / BENCHMARK_LOOPS as f64 * geo_std));
        times_geo_std = exp(sqrt(1_f64 / BENCHMARK_LOOPS as f64 * times_geo_std));

        debug!("Benchmark results for {}:", bench_name);
        debug!("(Geometric) Mean score: {}", geo_mean);
        debug!("Geometric Standard Deviation Score: {}", geo_std);
        debug!("Range: [{}, {}]", geo_mean / geo_std, geo_mean * geo_std);

        debug!("Timing results:");
        debug!("(Geometric) Mean time to completion: {}ms", times_geo_mean);
        debug!("Geometric Standard Deviation Time: {}", times_geo_std);
        debug!("Range(ms): [{}, {}]", times_geo_mean / times_geo_std, times_geo_mean * times_geo_std);

        (geo_mean, geo_std, times_geo_mean, times_geo_std)
    }

    fn initialise_board(_: &mut TimeTracking, _: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
        Ok(Vec::new())
    }


    fn start_trigger(start_end: &mut TimeTracking, _: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
        start_end.0 = Instant::now();
        Ok(Vec::new())
    }

    fn stop_trigger(start_end: &mut TimeTracking, _: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
        start_end.1 = Instant::now();
        Ok(Vec::new())
    }
}
