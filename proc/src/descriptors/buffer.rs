use crate::{find_unique, get_unique};

#[derive(Clone)]
pub struct Buffer {
    pub kind: Kind,
    pub ty: syn::Type,
}

#[derive(Clone, Copy)]
pub enum Kind {
    Storage,
    Uniform,
}

enum AttributeArgument {
    Kind(Kind),
    Type(syn::Type),
}

pub(super) fn parse_buffer_attr(attr: &syn::Attribute) -> Option<Buffer> {
    if attr.path.get_ident().map_or(true, |i| i != "buffer") {
        return None;
    }

    let args = attr
        .parse_args_with(|stream: syn::parse::ParseStream<'_>| {
            Ok(if stream.is_empty() {
                Vec::new()
            } else {
                stream
                    .parse_terminated::<_, syn::Token![,]>(|stream| {
                        let ident = stream.parse::<syn::Ident>()?;

                        match ident {
                            ident if ident == "uniform" => {
                                Ok(AttributeArgument::Kind(Kind::Uniform))
                            }
                            ident if ident == "storage" => {
                                Ok(AttributeArgument::Kind(Kind::Storage))
                            }
                            ident if ident == "ty" => {
                                let _eq = stream.parse::<syn::Token![=]>()?;
                                let ty = stream.parse::<syn::Type>()?;
                                Ok(AttributeArgument::Type(ty))
                            }
                            _ => {
                                return Err(stream.error("Unrecognized argument"));
                            }
                        }
                    })?
                    .into_iter()
                    .collect()
            })
        })
        .unwrap();

    let kind = find_unique(
        args.iter().filter_map(|arg| match arg {
            AttributeArgument::Kind(kind) => Some(*kind),
            _ => None,
        }),
        "Expected at most one `uniform` or `storage` argument",
    )
    .unwrap_or(Kind::Uniform);

    let ty = get_unique(
        args.iter().filter_map(|arg| match arg {
            AttributeArgument::Type(path) => Some(path.clone()),
            _ => None,
        }),
        "Expected exactly one `type` argument",
    );

    Some(Buffer { kind, ty })
}
