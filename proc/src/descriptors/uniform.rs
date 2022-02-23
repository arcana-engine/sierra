use crate::{find_unique, kw, StructLayout};

#[derive(Clone, Copy)]
pub(super) struct Uniform {
    pub layout: StructLayout,
}

impl Uniform {
    #[inline]
    pub fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}

enum UniformArgument {
    Layout(StructLayout),
}

pub(super) fn parse_uniform_attr(attr: &syn::Attribute) -> syn::Result<Option<Uniform>> {
    match attr.path.get_ident() {
        Some(ident) if ident == "uniform" => {
            let mut layout = StructLayout::Std140;
            if !attr.tokens.is_empty() {
                let args = attr.parse_args_with(|stream: syn::parse::ParseStream| {
                    stream.parse_terminated::<_, syn::Token![,]>(
                        |stream: syn::parse::ParseStream| {
                            let lookahead1 = stream.lookahead1();

                            if lookahead1.peek(kw::std140) {
                                stream.parse::<kw::std140>()?;
                                Ok(UniformArgument::Layout(StructLayout::Std140))
                            } else if lookahead1.peek(kw::std430) {
                                stream.parse::<kw::std430>()?;
                                Ok(UniformArgument::Layout(StructLayout::Std430))
                            } else {
                                Err(lookahead1.error())
                            }
                        },
                    )
                })?;

                layout = find_unique(
                    args.iter().filter_map(|arg| match arg {
                        UniformArgument::Layout(layout) => Some(*layout),
                    }),
                    attr,
                    "Only one layout attribute expected",
                )?
                .unwrap_or(StructLayout::Std140);
            }

            Ok(Some(Uniform { layout }))
        }
        _ => Ok(None),
    }
}
