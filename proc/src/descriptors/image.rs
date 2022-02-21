use crate::find_unique;

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

    let args = attr.parse_args_with(|stream: syn::parse::ParseStream<'_>| {
        if stream.is_empty() {
            Ok(Default::default())
        } else {
            let args = stream.parse_terminated::<_, syn::Token![,]>(|stream| {
                let ident = stream.parse::<syn::Ident>()?;

                match ident {
                    ident if ident == "sampled" => Ok(Argument::Sampled),
                    ident if ident == "storage" => Ok(Argument::Storage),
                    _ => Err(stream.error("Unrecognized argument")),
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
        args.iter().map(|arg| match arg {
            Argument::Sampled => Kind::Sampled,
            Argument::Storage => Kind::Storage,
        }),
        attr,
        "Expected at most one `uniform` or `storage` argument",
    )?
    .unwrap_or(Kind::Sampled);

    Ok(Some(Image { kind }))
}
