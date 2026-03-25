use crate::domain::{Cli, Subcommand};
use crate::error::{Error, ExecutionError};

/// Executes the parsed `Cli` command.
///
/// # Errors
/// Returns an `Error` if the command execution fails (e.g., simulated failure).
#[allow(clippy::needless_pass_by_value)]
pub fn execute(cli: Cli) -> Result<(), Error> {
    match cli {
        Cli::Run(Subcommand::SimulateFailure) => {
            Err(Error::CommandExecution(ExecutionError::SimulatedFailure))
        }
        Cli::Run(Subcommand::Show(cmd)) => crate::show::execute_show(
            &cmd,
            std::io::stdin(),
            std::io::stdout(),
            crate::show::serialize_document,
        )
        .map_err(|e| Error::CommandExecution(ExecutionError::Show(e))),
        Cli::Run(Subcommand::Apply(cmd, doc_path)) => {
            match crate::apply::execute_apply(&cmd, std::io::stdin(), std::io::stdout(), &doc_path)
            {
                Ok(true) => Ok(()),
                Ok(false) => Err(Error::CommandExecution(ExecutionError::Apply(
                    crate::apply::ApplyCommandError::ProposalRejected,
                ))),
                Err(e) => Err(Error::CommandExecution(ExecutionError::Apply(e))),
            }
        }
        Cli::Run(_) | Cli::Help(_) | Cli::Version(_) | Cli::Bare => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Depth;

    #[test]
    fn execute_returns_success_when_minimum_boundary() -> Result<(), String> {
        let cli = Cli::Run(Subcommand::ComplexState {
            depth: Depth::try_new(0).map_err(|e| e.to_string())?,
        });
        let result = execute(cli);
        assert_eq!(result, Ok(()));
        Ok(())
    }

    #[test]
    fn execute_returns_success_when_maximum_valid_boundary() -> Result<(), String> {
        let cli = Cli::Run(Subcommand::ComplexState {
            depth: Depth::try_new(254).map_err(|e| e.to_string())?,
        });
        let result = execute(cli);
        assert_eq!(result, Ok(()));
        Ok(())
    }

    #[test]
    fn execute_returns_success_when_bare_cli() {
        let cli = Cli::Bare;
        let result = execute(cli);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn execute_returns_success_when_help_state() {
        let cli = Cli::Help("some help".to_string());
        let result = execute(cli);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn execute_returns_success_when_valid_subcommand() {
        let cli = Cli::Run(Subcommand::ValidCommand);
        let result = execute(cli);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn execute_returns_error_when_simulate_failure_state() {
        let cli = Cli::Run(Subcommand::SimulateFailure);
        let result = execute(cli);
        assert_eq!(
            result,
            Err(Error::CommandExecution(ExecutionError::SimulatedFailure))
        );
    }

    // =========================================================================
    // BEHAVIOR 80: execute dispatches to execute_apply when Subcommand::Apply
    // =========================================================================

    #[test]
    fn execute_dispatches_to_execute_apply_when_subcommand_is_apply() {
        let cmd = crate::apply::ApplyCommand {
            input_source: crate::apply::ApplySource::File(std::path::PathBuf::from(
                "/nonexistent/proposal.json",
            )),
        };
        let doc_path = std::path::PathBuf::from("/nonexistent/doc.json");
        let cli = Cli::Run(Subcommand::Apply(cmd, doc_path));
        let result = execute(cli);

        // The proposal file doesn't exist, so execute_apply returns InputFileNotFound.
        // Proving the Apply arm was reached and delegated correctly.
        match result {
            Err(Error::CommandExecution(ExecutionError::Apply(
                crate::apply::ApplyCommandError::InputFileNotFound(_),
            ))) => {}
            other => panic!("expected ExecutionError::Apply(InputFileNotFound), got: {other:?}"),
        }
    }
}
