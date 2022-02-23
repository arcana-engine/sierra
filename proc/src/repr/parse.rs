use crate::{kw, StructLayout};

pub(super) struct Input {
    pub item_struct: syn::ItemStruct,
    pub layouts: Vec<StructLayout>,
}

enum ReprAttribute {
    Layout(StructLayout),
}

pub(super) fn parse(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> syn::Result<Input> {
    let item_struct = syn::parse::<syn::ItemStruct>(item)?;

    match &item_struct.fields {
        syn::Fields::Unit => {}
        syn::Fields::Named(_) => {}
        syn::Fields::Unnamed(fields) => {
            return Err(syn::Error::new_spanned(
                fields,
                "Tuple structs are not supported",
            ));
        }
    }

    let attrs = syn::parse::Parser::parse(
        |stream: syn::parse::ParseStream| {
            stream.parse_terminated::<_, syn::Token![,]>(|stream: syn::parse::ParseStream| {
                let lookahead1 = stream.lookahead1();
                if lookahead1.peek(kw::std140) {
                    stream.parse::<kw::std140>()?;
                    Ok(ReprAttribute::Layout(StructLayout::Std140))
                } else if lookahead1.peek(kw::std430) {
                    stream.parse::<kw::std430>()?;
                    Ok(ReprAttribute::Layout(StructLayout::Std430))
                } else {
                    Err(lookahead1.error())
                }
            })
        },
        attr,
    )?;

    let layouts = attrs
        .iter()
        .map(|attr| match attr {
            ReprAttribute::Layout(layout) => *layout,
        })
        .collect::<Vec<_>>();

    Ok(Input {
        item_struct,
        layouts,
    })
}
