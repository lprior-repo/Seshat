#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub struct Outbox;

impl Outbox {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Outbox {
    fn default() -> Self {
        Self::new()
    }
}
