#[derive(Clone, Copy)]
pub struct Uniform;

pub(super) fn parse_uniform_attr(attr: &syn::Attribute) -> Option<Uniform> {
    if attr.path.get_ident().map_or(true, |i| i != "uniform") {
        return None;
    }

    Some(Uniform)
}
