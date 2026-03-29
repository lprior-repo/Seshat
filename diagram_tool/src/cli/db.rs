#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

pub struct Db;

impl Db {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Db {
    fn default() -> Self {
        Self::new()
    }
}
