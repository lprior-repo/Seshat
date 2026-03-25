#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let args: Vec<String> = s.split_whitespace().map(|s| s.to_string()).collect();
        match seshat_cli::parse_args(args) {
            Ok(_) => (),
            Err(_) => (),
        }
    }
});
