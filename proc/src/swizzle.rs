use proc_macro2::{Span, TokenStream};

pub fn swizzle(tokens: proc_macro::TokenStream) -> TokenStream {
    match try_swizzle(tokens) {
        Ok(output) => output,
        Err(err) => err.to_compile_error(),
    }
}

fn try_swizzle(tokens: proc_macro::TokenStream) -> Result<TokenStream, syn::Error> {
    let ident = tokens.to_string();

    if ident.len() > 4 {
        return Err(syn::Error::new(
            Span::call_site(),
            "Swizzle must have at most 4 components",
        ));
    }

    let ident_bytes = ident.as_bytes();
    let r = ident_bytes.get(0).copied().unwrap_or(b'I');
    let g = ident_bytes.get(1).copied().unwrap_or(b'I');
    let b = ident_bytes.get(2).copied().unwrap_or(b'I');
    let a = ident_bytes.get(3).copied().unwrap_or(b'I');

    let comp = |c: u8| match c {
        b'0' => Ok("Zero"),
        b'1' => Ok("One"),
        b'R' | b'r' => Ok("R"),
        b'G' | b'g' => Ok("G"),
        b'B' | b'b' => Ok("B"),
        b'A' | b'a' => Ok("A"),
        b'I' | b'i' => Ok("Identity"),
        _ => Err(syn::Error::new(
            Span::call_site(),
            "Swizzle components must be 0, 1, r, g, b, a or i. Case-insensitive",
        )),
    };

    let output = format!(
        "::sierra::ComponentMapping {{r: ::sierra::Swizzle::{},g: ::sierra::Swizzle::{},b: ::sierra::Swizzle::{},a: ::sierra::Swizzle::{},}}",
        comp(r)?,
        comp(g)?,
        comp(b)?,
        comp(a)?,
    );

    Ok(output.parse().unwrap())
}
