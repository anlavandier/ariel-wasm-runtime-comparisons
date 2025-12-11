
#[macro_export]
macro_rules! benchmark_name {
    () => {
        env!("BENCHMARK")
    };
}

#[macro_export]
macro_rules! benchmark_file {
    () => {
        env!("BENCHMARK_PATH")
    }
}