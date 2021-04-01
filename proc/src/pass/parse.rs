use {
    crate::{find_unique_attribute, take_attributes},
    std::convert::TryFrom as _,
    syn::spanned::Spanned as _,
};

pub struct Input {
    pub item_struct: syn::ItemStruct,
    pub attachments: Vec<Attachment>,
    pub subpasses: Vec<Subpass>,
}

pub enum LoadOp {
    Load,
    Clear,
    DontCare,
}

pub enum StoreOp {
    Store,
    DontCare,
}

pub struct Attachment {
    pub member: syn::Member,
    pub ty: syn::Type,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
}

pub struct Subpass {
    pub colors: Vec<syn::Member>,
    pub depth: Option<syn::Member>,
}

pub fn parse(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> syn::Result<Input> {
    assert!(attr.is_empty());

    let mut item_struct = syn::parse::<syn::ItemStruct>(item)?;

    let mut attachments = Vec::with_capacity(item_struct.fields.len());

    for (i, f) in item_struct.fields.iter_mut().enumerate() {
        let i = match u32::try_from(i) {
            Ok(i) => i,
            Err(_) => {
                return Err(syn::Error::new_spanned(f, "Too many fields"));
            }
        };

        attachments.push(parse_attachment(f, i)?);
    }

    let subpasses = take_attributes(&mut item_struct.attrs, parse_subpass_attr)?;

    Ok(Input {
        item_struct,
        attachments,
        subpasses,
    })
}

enum SubpassArg {
    Color { member: syn::Member },
    Depth { member: syn::Member },
}

fn parse_subpass_attr(attr: &syn::Attribute) -> syn::Result<Option<Subpass>> {
    match attr.path.get_ident() {
        Some(ident) if ident == "subpass" => {}
        _ => return Ok(None),
    }

    let args = attr
        .parse_args_with(|stream: syn::parse::ParseStream<'_>| {
            stream.parse_terminated::<_, syn::Token![,]>(|stream: syn::parse::ParseStream<'_>| {
                match stream.parse::<syn::Ident>()? {
                    i if i == "color" => {
                        let _assign = stream.parse::<syn::Token![=]>()?;
                        let member = stream.parse::<syn::Member>()?;
                        Ok(SubpassArg::Color { member })
                    }
                    i if i == "depth" => {
                        let _assign = stream.parse::<syn::Token![=]>()?;
                        let member = stream.parse::<syn::Member>()?;
                        Ok(SubpassArg::Depth { member })
                    }
                    ident => Err(stream.error(format!("Unrecognized subpass argument {}", ident))),
                }
            })
        })
        .unwrap();

    let mut colors = Vec::new();
    let mut depth = None;

    for arg in args {
        match arg {
            SubpassArg::Color { member } => colors.push(member),
            SubpassArg::Depth { member } => {
                if depth.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "At most one `depth` argument for `subpass` attribute can be specified",
                    ));
                }

                depth = Some(member);
            }
        }
    }

    Ok(Some(Subpass { colors, depth }))
}

fn parse_attachment(field: &mut syn::Field, field_index: u32) -> syn::Result<Attachment> {
    let load_op = find_unique_attribute(
        &mut field.attrs,
        parse_load_attr,
        "At most one `clear` or `load` attribute",
    )?
    .unwrap_or(LoadOp::DontCare);

    let store_op = find_unique_attribute(
        &mut field.attrs,
        parse_store_attr,
        "At most one `clear` or `load` attribute",
    )?
    .unwrap_or(StoreOp::DontCare);

    let member = match field.ident.as_ref() {
        None => syn::Member::Unnamed(syn::Index {
            index: field_index,
            span: field.span(),
        }),
        Some(field_ident) => syn::Member::Named(field_ident.clone()),
    };

    Ok(Attachment {
        ty: field.ty.clone(),
        member,
        load_op,
        store_op,
    })
}

fn parse_load_attr(attr: &syn::Attribute) -> syn::Result<Option<LoadOp>> {
    match attr.path.get_ident() {
        Some(i) if i == "clear" => {
            if attr.tokens.is_empty() {
                Ok(Some(LoadOp::Clear))
            } else {
                Err(syn::Error::new_spanned(
                    attr,
                    "`clear` attribute does not accept arguments",
                ))
            }
        }
        Some(i) if i == "load" => {
            if attr.tokens.is_empty() {
                Ok(Some(LoadOp::Load))
            } else {
                Err(syn::Error::new_spanned(
                    attr,
                    "`load` attribute does not accept arguments",
                ))
            }
        }
        _ => Ok(None),
    }
}
fn parse_store_attr(attr: &syn::Attribute) -> syn::Result<Option<StoreOp>> {
    match attr.path.get_ident() {
        Some(i) if i == "store" => {
            if attr.tokens.is_empty() {
                Ok(Some(StoreOp::Store))
            } else {
                Err(syn::Error::new_spanned(
                    attr,
                    "`store` attribute does not accept arguments",
                ))
            }
        }
        _ => Ok(None),
    }
}
