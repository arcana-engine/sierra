use crate::{find_unique, get_unique};

#[derive(Clone)]
pub struct Buffer {
    pub kind: Kind,
    pub ty: syn::Type,
}

impl Buffer {
    #[inline]
    pub fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub enum Kind {
    Storage,
    Uniform,
}

enum Argument {
    Kind(Kind),
    Type(syn::Type),
}

pub(super) fn parse_buffer_attr(attr: &syn::Attribute) -> syn::Result<Option<Buffer>> {
    if attr.path.get_ident().map_or(true, |i| i != "buffer") {
        return Ok(None);
    }

    let args = attr.parse_args_with(|stream: syn::parse::ParseStream<'_>| {
        if stream.is_empty() {
            Ok(Default::default())
        } else {
            let args = stream.parse_terminated::<_, syn::Token![,]>(|stream| {
                let ident = stream.parse::<syn::Ident>()?;

                match ident {
                    ident if ident == "uniform" => Ok(Argument::Kind(Kind::Uniform)),
                    ident if ident == "storage" => Ok(Argument::Kind(Kind::Storage)),
                    ident if ident == "ty" => {
                        let _eq = stream.parse::<syn::Token![=]>()?;
                        let ty = stream.parse::<syn::Type>()?;

                        Ok(Argument::Type(ty))
                    }
                    _ => {
                        return Err(stream.error("Unrecognized argument"));
                    }
                }
            })?;

            if !stream.is_empty() {
                Err(stream.error("Single member is expected in arguments"))
            } else {
                Ok(args)
            }
        }
    })?;

    let kind = find_unique(
        args.iter().filter_map(|arg| match arg {
            Argument::Kind(kind) => Some(*kind),
            _ => None,
        }),
        attr,
        "Expected at most one `uniform` or `storage` argument",
    )?
    .unwrap_or(Kind::Uniform);

    let ty = get_unique(
        args.iter().filter_map(|arg| match arg {
            Argument::Type(path) => Some(path.clone()),
            _ => None,
        }),
        attr,
        "Expected exactly one `type` argument",
    )?;

    Ok(Some(Buffer { kind, ty }))
}
