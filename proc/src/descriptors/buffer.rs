use crate::{find_unique, kw};

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

    let mut texel = false;
    let mut kind = Kind::Uniform;

    if !attr.tokens.is_empty() {
        let args = attr.parse_args_with(|stream: syn::parse::ParseStream<'_>| {
            let args = stream.parse_terminated::<_, syn::Token![,]>(|stream| {
                let lookahead1 = stream.lookahead1();

                if lookahead1.peek(kw::uniform) {
                    stream.parse::<kw::uniform>()?;
                    Ok(Argument::Uniform)
                } else if lookahead1.peek(kw::storage) {
                    stream.parse::<kw::storage>()?;
                    Ok(Argument::Storage)
                } else if lookahead1.peek(kw::texel) {
                    stream.parse::<kw::texel>()?;
                    Ok(Argument::Texel)
                } else {
                    Err(lookahead1.error())
                }
            })?;

            if !stream.is_empty() {
                Err(stream.error("Single member is expected in arguments"))
            } else {
                Ok(args)
            }
        })?;

        texel = find_unique(
            args.iter().filter_map(|arg| match arg {
                Argument::Texel => Some(()),
                _ => None,
            }),
            attr,
            "Expected at most one `uniform` or `storage` argument",
        )?
        .is_some();

        kind = find_unique(
            args.iter().filter_map(|arg| match arg {
                Argument::Uniform => Some(Kind::Uniform),
                Argument::Storage => Some(Kind::Storage),
                _ => None,
            }),
            attr,
            "Expected at most one `uniform` or `storage` argument",
        )?
        .unwrap_or(Kind::Uniform);
    }

    Ok(Some(Buffer { kind, texel }))
}
