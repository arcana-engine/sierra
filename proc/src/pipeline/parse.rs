use crate::{
    find_unique, get_unique_attribute, kw,
    stage::{take_stages, Stage},
    StructLayout,
};

pub(super) struct Input {
    pub item_struct: syn::ItemStruct,
    pub sets: Vec<Set>,
    pub push_constants: Vec<PushConstants>,
}

pub(super) struct Set {
    pub field: syn::Field,
}

pub(super) struct PushConstants {
    pub field: syn::Field,
    pub stages: Vec<Stage>,
    pub layout: StructLayout,
}

enum LayoutField {
    Set(Set),
    PushConstants(PushConstants),
}

pub(super) fn parse(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> syn::Result<Input> {
    if !attr.is_empty() {
        return Err(syn::Error::new_spanned(
            proc_macro2::TokenStream::from(attr),
            "#[pipeline] attribute does not accept arguments",
        ));
    }

    let mut item_struct =
        syn::parse::<syn::ItemStruct>(item).expect("`#[pipeline]` can be applied only to structs");

    let mut sets = Vec::new();
    let mut push_constants = Vec::new();

    for field in item_struct.fields.iter_mut() {
        match parse_layout_field(field)? {
            LayoutField::Set(set) => sets.push(set),
            LayoutField::PushConstants(constants) => push_constants.push(constants),
        }
    }

    Ok(Input {
        item_struct,
        sets,
        push_constants,
    })
}

fn parse_layout_field(field: &mut syn::Field) -> syn::Result<LayoutField> {
    let ident = field
        .ident
        .clone()
        .expect("Only named struct are supported");

    let attr = get_unique_attribute(
        &mut field.attrs,
        parse_layout_attr,
        &ident,
        "Exactly one `set` or `push` attribute expected",
    )?;

    match attr {
        PipelineLaoyutAttribute::Set => Ok(LayoutField::Set(Set {
            field: field.clone(),
        })),
        PipelineLaoyutAttribute::PushConstants { layout } => {
            let stages = take_stages(&mut field.attrs)?;

            Ok(LayoutField::PushConstants(PushConstants {
                field: field.clone(),
                stages,
                layout,
            }))
        }
    }
}

enum PipelineLaoyutAttribute {
    Set,
    PushConstants { layout: StructLayout },
}

enum PushConstantArgument {
    Layout(StructLayout),
}

fn parse_layout_attr(attr: &syn::Attribute) -> syn::Result<Option<PipelineLaoyutAttribute>> {
    match attr.path.get_ident() {
        Some(ident) if ident == "set" => {
            if attr.tokens.is_empty() {
                Ok(Some(PipelineLaoyutAttribute::Set))
            } else {
                Err(syn::Error::new_spanned(
                    attr,
                    "`set` attribute does not accept arguments",
                ))
            }
        }
        Some(ident) if ident == "push" => {
            let mut layout = StructLayout::Std140;

            if !attr.tokens.is_empty() {
                let args = attr.parse_args_with(|stream: syn::parse::ParseStream<'_>| {
                    stream.parse_terminated::<_, syn::Token![,]>(
                        |stream: syn::parse::ParseStream<'_>| {
                            let lookahead1 = stream.lookahead1();
                            if lookahead1.peek(kw::std140) {
                                stream.parse::<kw::std140>()?;
                                Ok(PushConstantArgument::Layout(StructLayout::Std140))
                            } else if lookahead1.peek(kw::std430) {
                                stream.parse::<kw::std430>()?;
                                Ok(PushConstantArgument::Layout(StructLayout::Std430))
                            } else {
                                Err(lookahead1.error())
                            }
                        },
                    )
                })?;

                layout = find_unique(
                    args.iter().filter_map(|arg| match arg {
                        PushConstantArgument::Layout(layout) => Some(*layout),
                    }),
                    attr,
                    "Only one strucutre layout can be specifeid",
                )?
                .unwrap_or(StructLayout::Std140);
            }

            Ok(Some(PipelineLaoyutAttribute::PushConstants { layout }))
        }
        _ => Ok(None),
    }
}
