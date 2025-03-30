#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => { println!($($arg)*) };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {};
}
