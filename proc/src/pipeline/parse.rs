use crate::find_unique_attribute;

pub struct Input {
    pub item_struct: syn::ItemStruct,
    pub sets: Vec<Set>,
}

pub struct Set {
    pub ident: syn::Ident,
    pub ty: syn::Type,
}

pub fn parse(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> syn::Result<Input> {
    if !attr.is_empty() {
        return Err(syn::Error::new_spanned(
            proc_macro2::TokenStream::from(attr),
            "#[pipeline] attribute does not accept arguments",
        ));
    }

    let mut item_struct =
        syn::parse::<syn::ItemStruct>(item).expect("`#[pipeline]` can be applied only to structs");

    let mut sets = Vec::new();

    for field in item_struct.fields.iter_mut() {
        match parse_set_field(field)? {
            None => {}
            Some(set) => sets.push(set),
        }
    }

    Ok(Input { item_struct, sets })
}

fn parse_set_field(field: &mut syn::Field) -> syn::Result<Option<Set>> {
    let attr = find_unique_attribute(
        &mut field.attrs,
        parse_set_attr,
        "At most one `set` attribute",
    )?;

    if let Some(SetAttribute) = attr {
        let ident = field
            .ident
            .clone()
            .expect("Only named struct are supported");

        Ok(Some(Set {
            ident,
            ty: field.ty.clone(),
        }))
    } else {
        Ok(None)
    }
}

struct SetAttribute;

fn parse_set_attr(attr: &syn::Attribute) -> syn::Result<Option<SetAttribute>> {
    match attr.path.get_ident() {
        Some(ident) if ident == "set" => {
            if attr.tokens.is_empty() {
                Ok(Some(SetAttribute))
            } else {
                Err(syn::Error::new_spanned(
                    attr,
                    "`set` attribute does not accept arguments",
                ))
            }
        }
        _ => Ok(None),
    }
}
