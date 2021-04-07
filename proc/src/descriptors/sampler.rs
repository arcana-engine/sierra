pub struct Sampler;

impl Sampler {
    #[inline]
    pub fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}

pub(super) fn parse_sampler_attr(attr: &syn::Attribute) -> syn::Result<Option<Sampler>> {
    if attr.path.get_ident().map_or(true, |i| i != "sampler") {
        return Ok(None);
    }

    if !attr.tokens.is_empty() {
        return Err(syn::Error::new_spanned(
            attr,
            "`sampler` attribute does not accept arguments",
        ));
    }

    Ok(Some(Sampler))
}
