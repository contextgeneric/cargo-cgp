//! Attribute macros that know nothing of CGP, for the UI fixtures that pin how the tool leaves a
//! plain Rust crate's diagnostics alone. Each expansion fails the way a CGP macro's can, so rustc
//! makes the suggestion the tool drops in a crate that uses CGP.

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

/// Re-emit a struct with a generated lifetime parameter, `'__a__`, that its own fields do not use,
/// so a field the user wrote with an elided lifetime makes rustc suggest the generated one. The
/// struct's fields must end with a comma.
#[proc_macro_attribute]
pub fn generated_lifetime(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut out = Vec::new();
    let mut tokens = item.into_iter();
    while let Some(token) = tokens.next() {
        let is_struct = matches!(&token, TokenTree::Ident(ident) if ident.to_string() == "struct");
        out.push(token);
        if !is_struct {
            continue;
        }
        out.extend(tokens.next());
        out.extend(parse("<'__a__>"));
        if let Some(TokenTree::Group(fields)) = tokens.next() {
            let mut stream: Vec<TokenTree> = fields.stream().into_iter().collect();
            stream.extend(parse("__marker: ::core::marker::PhantomData<&'__a__ ()>,"));
            let mut group = Group::new(Delimiter::Brace, stream.into_iter().collect());
            group.set_span(fields.span());
            out.push(TokenTree::Group(group));
        }
    }
    out.into_iter().collect()
}

/// Replace a function's body with `holder.name.as_mut()`, which returns `Option<&mut String>`; a
/// function declared to return `Option<&String>` then fails inside the generated body, whose tokens
/// carry the attribute's span, so rustc suggests a conversion right after the attribute.
#[proc_macro_attribute]
pub fn generated_body(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item.into_iter()
        .map(|token| match token {
            TokenTree::Group(body) if body.delimiter() == Delimiter::Brace => {
                TokenTree::Group(Group::new(Delimiter::Brace, parse("holder.name.as_mut()")))
            }
            other => other,
        })
        .collect()
}

fn parse(source: &str) -> TokenStream {
    source.parse().expect("a fixed token string parses")
}
