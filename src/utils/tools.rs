#![allow(non_snake_case)]

use dirs;
use rand::{distributions::Slice, thread_rng, Rng};

// Exit process, default code is 0
pub fn exit(code: i32) {
    std::process::exit(code);
}

pub fn sleep(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

pub fn printVersion() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

pub fn getHomeDir() -> String {
    let home = dirs::home_dir().unwrap();
    let home_str = home.to_str().unwrap();
    return home_str.to_string();
}

// Generate random string with length and complexity
// length: length of the string
// complex: include special characters or not
// return: random string
pub fn getRandomString(length: usize, complex: bool) -> String {
    let mut charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    if complex {
        charset = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+-/=?@#$%^&";
    }

    let random_string: String = thread_rng()
        .sample_iter(Slice::new(charset).unwrap())
        .take(length)
        .map(|&c| c as char)
        .collect();
    return random_string;
}

pub fn getUuid() -> String {
    return uuid::Uuid::new_v4().to_string();
}
