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
    /// The module or item path to narrow the expansion to, from `--cgp-expand-item=<path>`.
    /// `None` expands the whole crate.
    pub item: Option<String>,
}

/// Take the expand-mode flags out of `args`, returning the request they carry.
///
/// Both flags are **removed**, because neither is a flag of rustc's and the compiler would reject
/// them. Only the `<flag>=<value>` form is accepted: a spaced value would be indistinguishable from
/// a following rustc argument, so the front-end always joins it with `=`.
///
/// The item flag is taken whether or not the mode flag is present, so a stray one can never reach
/// the compiler.
pub fn take_expand_request(
    args: &mut Vec<String>,
    flag: &str,
    item_flag: &str,
) -> Option<ExpandRequest> {
    let output = take_flag_value(args, flag);
    let item = take_flag_value(args, item_flag).filter(|item| !item.is_empty());

    output
        .filter(|output| !output.is_empty())
        .map(|output| ExpandRequest { output, item })
}

/// Remove `flag=<value>` from `args` and return the value.
fn take_flag_value(args: &mut Vec<String>, flag: &str) -> Option<String> {
    let prefix = format!("{flag}=");
    let index = args.iter().position(|arg| arg.starts_with(&prefix))?;
    let arg = args.remove(index);
    Some(arg[prefix.len()..].to_owned())
}
