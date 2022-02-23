use crate::{find_unique, kw};

pub enum Layout {
    Expr(Box<syn::Expr>),
    Member(Box<syn::Member>),
}

pub struct Image {
    pub kind: Kind,
    pub layout: Option<Layout>,
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
    Layout(Layout),
}

pub(super) fn parse_image_attr(attr: &syn::Attribute) -> syn::Result<Option<Image>> {
    if attr.path.get_ident().map_or(true, |i| i != "image") {
        return Ok(None);
    }

    let mut kind = Kind::Sampled;
    let mut layout = None;

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
                } else if lookahead1.peek(kw::layout) {
                    stream.parse::<kw::layout>()?;
                    stream.parse::<syn::Token![=]>()?;

                    let layout = if stream.peek(syn::Token![const]) {
                        let _const = stream.parse::<syn::Token![const]>()?;
                        let expr = stream.parse::<syn::Expr>()?;
                        Layout::Expr(Box::new(expr))
                    } else {
                        let member = stream.parse::<syn::Member>()?;
                        Layout::Member(Box::new(member))
                    };

                    Ok(Argument::Layout(layout))
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
            args.iter().filter_map(|arg| match arg {
                Argument::Sampled => Some(Kind::Sampled),
                Argument::Storage => Some(Kind::Storage),
                _ => None,
            }),
            attr,
            "Expected at most one `uniform` or `storage` argument",
        )?
        .unwrap_or(Kind::Sampled);

        layout = find_unique(
            args.into_iter().filter_map(|arg| match arg {
                Argument::Layout(layout) => Some(layout),
                _ => None,
            }),
            attr,
            "Expected at most one `uniform` or `storage` argument",
        )?;
    }

    Ok(Some(Image { kind, layout }))
}
