use super::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_args_never_panics(
        args in prop::collection::vec(any::<String>(), 0..100)
    ) {
        match parse_args(args.into_iter().map(|s: String| OsString::from(s))) {
            Ok(_) => prop_assert!(true),
            Err(e) => prop_assert!(matches!(e, Error::ArgumentParse(_) | Error::CommandExecution(_))),
        }
    }
}
