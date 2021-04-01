#[derive(Clone, Copy)]
pub struct Uniform;

pub(super) fn parse_uniform_attr(attr: &syn::Attribute) -> syn::Result<Option<Uniform>> {
    match attr.path.get_ident() {
        Some(ident) if ident == "uniform" => {
            if attr.tokens.is_empty() {
                Ok(Some(Uniform))
            } else {
                Err(syn::Error::new_spanned(
                    attr,
                    "`uniform` attribute does not accept any arguments",
                ))
            }
        }
        _ => Ok(None),
    }
}
