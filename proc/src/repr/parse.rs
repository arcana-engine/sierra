pub struct Field {
    pub ident: syn::Ident,
    pub ty: syn::Type,
}

pub struct Input {
    pub fields: Vec<Field>,
    pub item_struct: syn::ItemStruct,
}

pub fn parse(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> Input {
    assert!(attr.is_empty());

    let mut item_struct = syn::parse::<syn::ItemStruct>(item)
        .expect("`#[shader_struct]` can be applied only to structs");

    let fields: Vec<_> = item_struct
        .fields
        .iter_mut()
        .map(|field| Field {
            ty: field.ty.clone(),
            ident: field
                .ident
                .clone()
                .expect("Tuple structs are not supported"),
        })
        .collect();

    Input {
        fields,
        item_struct,
    }
}
