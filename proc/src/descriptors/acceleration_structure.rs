pub struct AccelerationStructure;

pub(super) fn parse_acceleration_structure_attr(
    attr: &syn::Attribute,
) -> syn::Result<Option<AccelerationStructure>> {
    if attr
        .path
        .get_ident()
        .map_or(true, |i| i != "acceleration_structure" && i != "tlas")
    {
        return Ok(None);
    }

    if !attr.tokens.is_empty() {
        return Err(syn::Error::new_spanned(
            attr,
            "`acceleration_structure` attribute does not accept arguments",
        ));
    }

    Ok(Some(AccelerationStructure))
}
