#[macro_export]
macro_rules! time_it {
    ($context:literal, $($s:stmt);+ $(;)?) => {
        let timer = std::time::Instant::now();
        $(
            $s
        )*
        println!("[TIMEIT] {} took {:.2?}.", $context, timer.elapsed());
    };
}
