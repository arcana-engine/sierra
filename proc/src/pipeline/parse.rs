use crate::find_unique;

pub struct Input {
    pub item_struct: syn::ItemStruct,
    pub sets: Vec<Set>,
}

pub struct Set {
    pub ident: syn::Ident,
    pub ty: syn::Type,
}

pub fn parse(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> Input {
    assert!(attr.is_empty());

    let mut item_struct =
        syn::parse::<syn::ItemStruct>(item).expect("`#[pipeline]` can be applied only to structs");

    let mut sets = Vec::new();

    for field in item_struct.fields.iter_mut() {
        match parse_field_attrs(field) {
            None => {}
            Some(Field::Set(set)) => sets.push(set),
        }
    }

    Input { item_struct, sets }
}

enum Field {
    Set(Set),
}

enum FieldAttribute {
    Set,
}

fn parse_field_attrs(field: &mut syn::Field) -> Option<Field> {
    let (attr, index) = find_unique(
        field
            .attrs
            .iter()
            .enumerate()
            .filter_map(|(index, attr)| parse_input_field_attr(attr).map(|attr| (attr, index))),
        "At most one `set` attribute",
    )?;

    field.attrs.swap_remove(index);

    let ident = field
        .ident
        .clone()
        .expect("Only named struct are supported");

    Some(match attr {
        FieldAttribute::Set => Field::Set(Set {
            ident,
            ty: field.ty.clone(),
        }),
    })
}

fn parse_input_field_attr(attr: &syn::Attribute) -> Option<FieldAttribute> {
    on_first!(parse_set_attr(attr).map(|SetAttribute| FieldAttribute::Set));
    None
}

struct SetAttribute;

fn parse_set_attr(attr: &syn::Attribute) -> Option<SetAttribute> {
    if attr.path.get_ident().map_or(true, |i| i != "set") {
        return None;
    }

    assert!(
        attr.tokens.is_empty(),
        "`set` attribute does not support any arguments"
    );

    Some(SetAttribute)
}
