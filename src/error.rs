//! Error handling.

use failure::Error;
use failure::Fail;

/// Result type that the tool internally uses.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Prints the given error to the standard error output.
pub fn print_error(err: &Error) {
    eprintln!("error: {}", err);

    let fail: &dyn Fail = err.as_fail();
    for cause in fail.iter_causes() {
        eprintln!("  cause: {}", cause);
    }

    if let Some(bt) = fail.backtrace() {
        eprintln!("backtrace:");
        eprintln!("{}", bt);
    }
}
