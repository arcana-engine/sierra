use crate::find_unique;

#[derive(Clone)]
pub struct Buffer {
    pub texel: bool,
    pub kind: Kind,
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
    Uniform,
    Storage,
    Texel,
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
                    ident if ident == "uniform" => Ok(Argument::Uniform),
                    ident if ident == "storage" => Ok(Argument::Storage),
                    ident if ident == "texel" => Ok(Argument::Texel),
                    _ => Err(stream.error(format!("Unrecognized argument '{}'", ident))),
                }
            })?;

            if !stream.is_empty() {
                Err(stream.error("Single member is expected in arguments"))
            } else {
                Ok(args)
            }
        }
    })?;

    let texel = find_unique(
        args.iter().filter_map(|arg| match arg {
            Argument::Texel => Some(()),
            _ => None,
        }),
        attr,
        "Expected at most one `uniform` or `storage` argument",
    )?;

    let kind = find_unique(
        args.iter().filter_map(|arg| match arg {
            Argument::Uniform => Some(Kind::Uniform),
            Argument::Storage => Some(Kind::Storage),
            _ => None,
        }),
        attr,
        "Expected at most one `uniform` or `storage` argument",
    )?
    .unwrap_or(Kind::Uniform);

    Ok(Some(Buffer {
        kind,
        texel: texel.is_some(),
    }))
}
