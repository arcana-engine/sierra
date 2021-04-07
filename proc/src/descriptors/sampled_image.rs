pub struct SampledImage;

impl SampledImage {
    #[inline]
    pub fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}

pub(super) fn parse_sampled_image_attr(attr: &syn::Attribute) -> syn::Result<Option<SampledImage>> {
    if attr.path.get_ident().map_or(true, |i| i != "sampled_image") {
        return Ok(None);
    }

    if !attr.tokens.is_empty() {
        return Err(syn::Error::new_spanned(
            attr,
            "`sampled_image` attribute does not accept arguments",
        ));
    }

    Ok(Some(SampledImage))
}
