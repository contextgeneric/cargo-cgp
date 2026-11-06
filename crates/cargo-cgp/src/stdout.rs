//! Writing the front-end's output to stdout, whose reader may have closed the pipe.

use std::io::{self, Write};

/// Write `text` to `out`. A reader that closed the pipe, as `head` does once it has enough, got
/// what it wanted, so that is not a failure; any other write error is returned.
pub fn write_output(out: &mut impl Write, text: &str) -> io::Result<()> {
    match out.write_all(text.as_bytes()).and_then(|()| out.flush()) {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result,
    }
}

/// Print `text` and a newline to stdout through [`write_output`], the way every answer the
/// front-end prints itself (its help texts and its version) goes out.
pub fn print_line(text: &str) -> io::Result<()> {
    write_output(&mut io::stdout(), &format!("{text}\n"))
}
