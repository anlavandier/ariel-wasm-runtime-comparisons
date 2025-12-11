use wasm::{validate, RuntimeInstance, Value, HaltExecutionError};

extern crate alloc;
use alloc::vec::Vec;

#[cfg(feature = "minimal")]
fn extra(_: &mut (), _: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
    Ok(Vec::from_iter(core::iter::once(Value::I32(100))))
}

#[cfg(feature = "minimal")]
pub fn run_wasm() {
    let wasm_bytes = include_bytes!("../payload-wasm/input.wasm");

    let validation_info = validate(wasm_bytes).unwrap();

    let mut instance = RuntimeInstance::new(());

    instance.add_host_function_typed::<(), u32>("host", "extra", extra).unwrap();

    instance.add_module("module", &validation_info).unwrap();

    let res: u32 = instance.invoke_typed(
        &instance.get_function_by_name("module", "add_with_extra").unwrap(),
        (28, 16)).unwrap();
    assert_eq!(res, 28 + 16 + 100);
}


#[cfg(feature = "coremark")]
fn clock_ms(_: &mut (), _: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
    Ok(Vec::from_iter(core::iter::once(Value::I64(ariel_os::time::Instant::now().as_millis()))))
}

#[cfg(feature = "coremark")]
pub fn run_coremark() -> f32 {
    let wasm_bytes = include_bytes!(crate::benchmark_file!());

    let validation_info = validate(wasm_bytes).unwrap();

    let mut instance = RuntimeInstance::new(());

    instance.add_host_function_typed::<(), u64>("env", "clock_ms", clock_ms).unwrap();

    instance.add_module("module", &validation_info).unwrap();

    let res: f32 = instance.invoke_typed(
        &instance.get_function_by_name("module", "run").unwrap(), ()).unwrap();
    return res
}


#[cfg(feature = "embench-1")]
pub mod embench1 {
    use ariel_os::time::Instant;
    use libm::{pow, exp, log, sqrt};
    use ariel_os::debug::log::{info, debug, error};

    use super::*;
    use crate::{BENCH_SCORE, BENCHMARK_LOOPS, benchmark_name, benchmark_file};

    struct TimeTracking(Instant, Instant);

    use wasm::config::Config;

    impl Config for TimeTracking {}

    pub fn run_bench() {
        let bench_name = benchmark_name!();
        let wasm = include_bytes!(benchmark_file!());

        let validation_info = validate(wasm).unwrap();

        let mut instance = RuntimeInstance::new(TimeTracking(Instant::now(), Instant::now()));

        instance.add_host_function_typed::<(), ()>("env", "initialise_board", initialise_board).unwrap();
        instance.add_host_function_typed::<(), ()>("env", "start_trigger", start_trigger).unwrap();
        instance.add_host_function_typed::<(), ()>("env", "stop_trigger", stop_trigger).unwrap();

        instance.add_module("module", &validation_info).unwrap();

        let bench_function = instance.get_function_by_name("module", "__original_main").unwrap();
        debug!("Starting wasm app");
        let mut times_to_run = Vec::new();
        for i in 1..=BENCHMARK_LOOPS {
            debug!("Run {}", i);
            let correct = instance.invoke_typed(&bench_function, ()).unwrap();
            match correct {
                0 => {
                    let &TimeTracking(start, end) = instance.user_data();
                    // The relative speed is in milli seconds
                    times_to_run.push((end - start).as_millis());
                },
                _ => {
                    error!("Benchmarking went wrong for some reason, aborting");
                    return;
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

        info!("{}, {}, {}; {}, {}", bench_name, geo_mean, geo_std, times_geo_mean, times_geo_std);
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
