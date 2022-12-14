#[macro_export]
macro_rules! measure_time {
    ($code: block) => {{
        use std::time::Instant;
        let start = Instant::now();
        let result = $code;
        let elapsed = start.elapsed();
        elapsed
    }};
}