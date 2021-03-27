use crate::find_unique;

pub struct Field {
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub as_type: Option<syn::Type>,
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
        .map(|field| {
            let as_type = find_unique(
                field.attrs.iter().enumerate().filter_map(|(i, attr)| {
                    match attr.path.get_ident() {
                        Some(ident) if ident == "as_type" => {
                            let as_type = attr.parse_args::<syn::Type>().unwrap();
                            Some((i, as_type))
                        }
                        _ => None,
                    }
                }),
                "at most one attribute `as_type` can be specified",
            );

            let as_type = match as_type {
                Some((index, as_type)) => {
                    field.attrs.remove(index);
                    Some(as_type)
                }
                None => None,
            };

            Field {
                ty: field.ty.clone(),
                as_type,
                ident: field
                    .ident
                    .clone()
                    .expect("Tuple structs are not supported"),
            }
        })
        .collect();

    Input {
        fields,
        item_struct,
    }
}
