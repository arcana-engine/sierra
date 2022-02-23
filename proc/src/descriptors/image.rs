use crate::{find_unique, kw};

pub struct Image {
    pub kind: Kind,
}

impl Image {
    #[inline]
    pub fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub enum Kind {
    Sampled,
    Storage,
}

enum Argument {
    Sampled,
    Storage,
}

pub(super) fn parse_image_attr(attr: &syn::Attribute) -> syn::Result<Option<Image>> {
    if attr.path.get_ident().map_or(true, |i| i != "image") {
        return Ok(None);
    }

    let mut kind = Kind::Sampled;

    if !attr.tokens.is_empty() {
        let args = attr.parse_args_with(|stream: syn::parse::ParseStream<'_>| {
            let args = stream.parse_terminated::<_, syn::Token![,]>(|stream| {
                let lookahead1 = stream.lookahead1();

                if lookahead1.peek(kw::sampled) {
                    stream.parse::<kw::sampled>()?;
                    Ok(Argument::Sampled)
                } else if lookahead1.peek(kw::storage) {
                    stream.parse::<kw::storage>()?;
                    Ok(Argument::Storage)
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

        kind = find_unique(
            args.iter().map(|arg| match arg {
                Argument::Sampled => Kind::Sampled,
                Argument::Storage => Kind::Storage,
            }),
            attr,
            "Expected at most one `uniform` or `storage` argument",
        )?
        .unwrap_or(Kind::Sampled);
    }

    Ok(Some(Image { kind }))
}
