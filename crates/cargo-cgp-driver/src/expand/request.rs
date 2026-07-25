//! Recognizing the front-end's expand-mode flag.

/// Where the driver writes the finished expansion, when the front-end asked for one.
///
/// The request is the presence of `--cgp-expand=<path>` in the process arguments, which the
/// front-end appends after `cargo rustc`'s `--` so cargo puts it on exactly one target's rustc
/// invocation.
#[derive(Clone, Debug)]
pub struct ExpandRequest {
    /// The file the expansion is written to, for the front-end to read and print.
    pub output: String,
}

/// Take the expand-mode flag out of `args`, returning the request it carries.
///
/// The flag is **removed**, because it is no flag of rustc's and the compiler would reject it.
/// Only the `--cgp-expand=<path>` form is accepted: a spaced value would be indistinguishable
/// from a following rustc argument, so the front-end always joins it with `=`.
pub fn take_expand_request(args: &mut Vec<String>, flag: &str) -> Option<ExpandRequest> {
    let prefix = format!("{flag}=");
    let index = args.iter().position(|arg| arg.starts_with(&prefix))?;
    let arg = args.remove(index);
    let output = arg[prefix.len()..].to_owned();

    (!output.is_empty()).then_some(ExpandRequest { output })
}
