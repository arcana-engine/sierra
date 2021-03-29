pub struct AccelerationStructure;

pub(super) fn parse_acceleration_structure_attr(
    attr: &syn::Attribute,
) -> Option<AccelerationStructure> {
    if attr
        .path
        .get_ident()
        .map_or(true, |i| i != "acceleration_structure" && i != "tlas")
    {
        return None;
    }

    if !attr.tokens.is_empty() {
        panic!("`acceleration_structure` attribute does not support any arguments")
    }

    // let () = attr.parse_args_with(|stream: syn::parse::ParseStream<'_>| {
    //         if stream.is_empty() {
    //             Ok(())
    //         } else {
    //             Err(stream.error("`acceleration_structure` attribute does not support any arguments"))
    //         }
    //     }).unwrap();

    Some(AccelerationStructure)
}
