use chrono::Local;

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        let now = $crate::utils::timestamp();
        println!("INFO  [{}] {}", now, format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        let now = $crate::utils::timestamp();
        eprintln!("ERROR [{}] {}", now, format!($($arg)*));
    }};
}

pub fn timestamp() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
