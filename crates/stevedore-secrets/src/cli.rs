//! Driving an external store CLI.
//!
//! Both halves of a move run a vendor's own command-line tool, so the rules
//! that keep a secret out of the wrong channel live here once: stdout is
//! treated as vault contents and never logged, stderr is stripped of escape
//! codes before it reaches an error, and a secret is handed over on stdin
//! rather than in an argument.
//!
//! It is also the one launch point, so the policy that gives a store CLI an
//! owner-only data directory is applied here — see [`crate::dataroot`].

use std::{
    io::{ErrorKind, Write},
    process::{Child, Command, Output, Stdio},
};

use crate::{
    dataroot,
    error::{CliError, Error, Result},
};

/// Run `program` and return its stdout.
///
/// Stdin is closed, so a tool that would prompt for credentials fails cleanly
/// instead of hanging on a child process nobody can answer.
///
/// # Errors
///
/// [`CliError::NotFound`] when `program` isn't on `PATH`, [`CliError::Failed`]
/// on a non-zero exit, or [`Error::Io`].
pub(crate) fn run(program: &'static str, args: &[&str]) -> Result<Vec<u8>> {
    let child = spawn(program, args, Stdio::null())?;
    let output = child
        .wait_with_output()
        .map_err(|e| io_error("waiting for", program, e))?;
    check(program, args, output)
}

/// Run `program` with `input` on stdin and return its stdout.
///
/// This is how a secret reaches a store's CLI. A process's arguments are
/// readable by other processes on the machine; a pipe is not.
///
/// # Errors
///
/// As [`run`].
pub(crate) fn run_with_stdin(
    program: &'static str,
    args: &[&str],
    input: &[u8],
) -> Result<Vec<u8>> {
    let mut child = spawn(program, args, Stdio::piped())?;
    let mut pipe = child
        .stdin
        .take()
        .expect("stdin should be piped by `spawn`");
    let written = pipe.write_all(input);
    drop(pipe);

    let output = child
        .wait_with_output()
        .map_err(|e| io_error("waiting for", program, e))?;
    // A non-zero exit explains a broken pipe better than the write error does.
    let stdout = check(program, args, output)?;
    written.map_err(|e| io_error("writing to", program, e))?;
    Ok(stdout)
}

fn spawn(program: &'static str, args: &[&str], stdin: Stdio) -> Result<Child> {
    dataroot::prepare(program)?;
    Command::new(program)
        .args(args)
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => CliError::NotFound { program }.into(),
            _ => io_error("spawning", program, e),
        })
}

fn io_error(doing: &'static str, program: &'static str, source: std::io::Error) -> Error {
    Error::Io {
        doing,
        program,
        source,
    }
}

/// stdout may be a plaintext vault dump: never log it or attach it to an error.
fn check(program: &'static str, args: &[&str], output: Output) -> Result<Vec<u8>> {
    if output.status.success() {
        return Ok(output.stdout);
    }
    Err(CliError::Failed {
        program,
        args: args.join(" "),
        status: output.status.to_string(),
        stderr: strip_ansi(&String::from_utf8_lossy(&output.stderr))
            .trim()
            .to_owned(),
    }
    .into())
}

/// Remove ANSI escape sequences, which store CLIs write into their messages.
fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        // An ANSI escape ends at its first letter.
        for c in chars.by_ref() {
            if c.is_ascii_alphabetic() {
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MISSING: &str = "stevedore-no-such-program";

    #[test]
    fn strips_ansi_colour_codes() {
        let coloured = "\u{1b}[31merror: No matching item found\u{1b}[0m";
        assert_eq!(strip_ansi(coloured), "error: No matching item found");
    }

    #[test]
    fn leaves_plain_text_alone() {
        assert_eq!(strip_ansi("no escapes here"), "no escapes here");
    }

    #[test]
    fn a_missing_program_is_reported_by_name() {
        let err = run(MISSING, &["status"]).unwrap_err();
        assert!(matches!(
            err,
            Error::Cli(CliError::NotFound { program }) if program == MISSING
        ));
    }

    /// A directory is not executable, so spawning one fails for a reason other
    /// than not-found — the path that reaches [`Error::Io`].
    #[test]
    fn an_io_failure_names_the_operation_and_the_program() {
        let err = run("/", &["ignored"]).unwrap_err();
        let Error::Io { doing, program, .. } = &err else {
            panic!("expected an io error, got {err:?}");
        };
        assert_eq!((*doing, *program), ("spawning", "/"));
    }

    /// The variant says only what it uniquely knows; the operating system's own
    /// message stays in the source, where a reporter joins it with `: `.
    #[test]
    fn an_io_error_never_restates_its_source() {
        let err = run("/", &["ignored"]).unwrap_err();
        assert_eq!(format!("{err}"), "spawning `/`");

        let source = std::error::Error::source(&err).expect("an io error should keep its source");
        let source = source
            .downcast_ref::<std::io::Error>()
            .expect("the source should be the io error");
        assert_eq!(source.kind(), ErrorKind::PermissionDenied);
    }

    #[test]
    fn a_missing_program_fails_before_stdin_is_written() {
        let err = run_with_stdin(MISSING, &["create"], b"secret payload").unwrap_err();
        assert!(matches!(err, Error::Cli(CliError::NotFound { .. })));
    }

    #[test]
    fn stdin_reaches_the_program_and_stdout_comes_back() {
        let out = run_with_stdin("cat", &[], b"payload").expect("cat should run");
        assert_eq!(out, b"payload");
    }

    #[test]
    fn a_non_zero_exit_reports_the_program_and_its_stderr() {
        let err = run("sh", &["-c", "echo boom >&2; exit 3"]).unwrap_err();
        let Error::Cli(CliError::Failed {
            program,
            status,
            stderr,
            ..
        }) = err
        else {
            panic!("expected a failed command, got {err:?}");
        };
        assert_eq!(program, "sh");
        assert_eq!(stderr, "boom");
        assert!(
            status.contains('3'),
            "status should name the code: {status}"
        );
    }

    #[test]
    fn a_failure_never_echoes_what_was_written_to_stdin() {
        // The payload is the secret; only the tool's own diagnostic may surface.
        let err =
            run_with_stdin("sh", &["-c", "cat >/dev/null; exit 1"], b"SEKRET-MARKER").unwrap_err();
        assert!(
            !format!("{err}").contains("SEKRET-MARKER"),
            "Display: {err}"
        );
        assert!(
            !format!("{err:?}").contains("SEKRET-MARKER"),
            "Debug: {err:?}"
        );
    }
}
