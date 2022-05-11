use proc_easy::EasyAttributes;
use syn::spanned::Spanned;

use crate::layout::StructLayouts;

pub(super) struct Input {
    pub item_struct: syn::ItemStruct,
    pub layouts: StructLayouts,
}

proc_easy::easy_attributes! {
    @(sierra)
    struct ReprAttributes {
        layouts: StructLayouts,
    }
}

pub(super) fn parse(item: proc_macro::TokenStream) -> syn::Result<Input> {
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

    let attributes = ReprAttributes::parse(&item_struct.attrs, item_struct.span())?;

    Ok(Input {
        item_struct,
        layouts: attributes.layouts,
    })
}
