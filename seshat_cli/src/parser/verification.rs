#![allow(unexpected_cfgs)]

use super::*;

#[kani::proof]
#[test]
fn no_panic_in_parse_args() {
    let args = vec![OsString::from("arg1"), OsString::from("arg2")];
    match parse_args(args.into_iter()) {
        Ok(_) => assert!(true),
        Err(e) => assert!(matches!(
            e,
            Error::ArgumentParse(_) | Error::CommandExecution(_)
        )),
    }
}
