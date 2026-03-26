use crate::domain::{ShowCommand, ShowSource};

/// Maps the clap-parsed Show args into the domain `ShowCommand` type.
/// Pure mapping function — no I/O.
///
/// # Postconditions
/// - Returns `ShowCommand { source: ShowSource::File(path) }` when `file` is `Some`.
/// - Returns `ShowCommand { source: ShowSource::Stdin }` when `file` is `None`.
pub(crate) fn map_show_subcommand(file: Option<std::path::PathBuf>) -> ShowCommand {
    ShowCommand {
        source: file.map_or(ShowSource::Stdin, ShowSource::File),
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn map_show_subcommand_returns_file_source_when_path_is_some() {
        let path = PathBuf::from("/some/path.json");
        let result = map_show_subcommand(Some(path.clone()));
        assert_eq!(
            result,
            ShowCommand {
                source: ShowSource::File(path)
            }
        );
    }

    #[test]
    fn map_show_subcommand_returns_file_source_when_path_is_relative() {
        let path = PathBuf::from("rel/path.json");
        let result = map_show_subcommand(Some(path.clone()));
        assert_eq!(
            result,
            ShowCommand {
                source: ShowSource::File(path)
            }
        );
    }

    #[test]
    fn map_show_subcommand_returns_file_source_when_path_is_root() {
        let path = PathBuf::from("/");
        let result = map_show_subcommand(Some(path.clone()));
        assert_eq!(
            result,
            ShowCommand {
                source: ShowSource::File(path)
            }
        );
    }

    #[test]
    fn map_show_subcommand_returns_stdin_source_when_path_is_none() {
        let result = map_show_subcommand(None);
        assert_eq!(
            result,
            ShowCommand {
                source: ShowSource::Stdin
            }
        );
    }

    #[test]
    fn show_command_clone_produces_equal_value() {
        let cmd = ShowCommand {
            source: ShowSource::File(PathBuf::from("/a/b.json")),
        };
        let cloned = cmd.clone();
        assert_eq!(cloned, cmd);
    }

    #[test]
    fn show_command_debug_output_contains_type_and_variant_names() {
        let cmd = ShowCommand {
            source: ShowSource::Stdin,
        };
        let output = format!("{cmd:?}");
        assert!(output.contains("ShowCommand"));
        assert!(output.contains("Stdin"));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use std::path::PathBuf;

    proptest! {
        #[test]
        fn proptest_map_show_subcommand_never_panics_for_any_option_pathbuf(
            bytes in prop::option::of(prop::collection::vec(any::<u8>(), 0..256))
        ) {
            let path_opt = bytes.map(|b| {
                use std::ffi::OsString;
                #[cfg(unix)]
                {
                    use std::os::unix::ffi::OsStringExt;
                    PathBuf::from(OsString::from_vec(b))
                }
                #[cfg(not(unix))]
                {
                    PathBuf::from(String::from_utf8_lossy(&b).to_string())
                }
            });
            let is_some = path_opt.is_some();
            let result = map_show_subcommand(path_opt);
            if is_some {
                prop_assert!(matches!(result.source, ShowSource::File(_)));
            } else {
                prop_assert_eq!(result.source, ShowSource::Stdin);
            }
        }
    }
}

#[allow(unexpected_cfgs)]
#[cfg(kani)]
mod verification {
    use super::*;
    use std::path::PathBuf;

    #[kani::proof]
    fn verify_map_show_subcommand_is_structurally_total() {
        let has_file: bool = kani::any();
        let cmd = if has_file {
            map_show_subcommand(Some(PathBuf::from("/bounded/path.json")))
        } else {
            map_show_subcommand(None)
        };
        if has_file {
            assert!(matches!(cmd.source, ShowSource::File(_)));
        } else {
            assert!(matches!(cmd.source, ShowSource::Stdin));
        }
    }
}
