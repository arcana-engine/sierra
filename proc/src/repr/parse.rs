pub struct Field {
    pub ident: syn::Ident,
    pub ty: syn::Type,
}

pub struct Input {
    pub fields: Vec<Field>,
    pub item_struct: syn::ItemStruct,
}

pub fn parse(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> syn::Result<Input> {
    if !attr.is_empty() {
        return Err(syn::Error::new_spanned(
            proc_macro2::TokenStream::from(attr),
            "#[shader_repr] attribute does not accept arguments",
        ));
    }

    let item_struct = syn::parse::<syn::ItemStruct>(item)?;

    let fields = item_struct
        .fields
        .iter()
        .map(|field| {
            let ident = field
                .ident
                .clone()
                .ok_or_else(|| syn::Error::new_spanned(field, "Tuple structs are not supported"))?;

            Ok(Field {
                ty: field.ty.clone(),
                ident,
            })
        })
        .collect::<Result<Vec<_>, syn::Error>>()?;

    Ok(Input {
        fields,
        item_struct,
    })
}
