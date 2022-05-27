use proc_easy::{private::Spanned, EasyAttributes};

use crate::{kw, layout::StructLayout, stage::Stages};

pub(super) struct Input {
    pub item_struct: syn::ItemStruct,
    pub sets: Vec<Set>,
    pub push_constants: Vec<PushConstants>,
}

proc_easy::easy_argument_tuple! {
    struct Push {
        kw: kw::push,
        layout: Option<StructLayout>,
    }
}

proc_easy::easy_argument_group! {
    enum KindAttr {
        Set(kw::set),
        PushConstants(Push),
    }
}

proc_easy::easy_attributes! {
    @(sierra)
    struct FieldAttrs {
        kind: KindAttr,
        stages: Option<Stages>,
    }
}

pub(super) struct PushConstants {
    pub field: syn::Field,
    pub stages: Stages,
    pub layout: StructLayout,
}

pub(super) struct Set {
    pub field: syn::Field,
}

pub(super) fn parse(item: proc_macro::TokenStream) -> syn::Result<Input> {
    let item_struct =
        syn::parse::<syn::ItemStruct>(item).expect("`#[Pipeline]` can be derived only for structs");

    let mut sets = Vec::new();
    let mut push_constants = Vec::new();

    for field in &item_struct.fields {
        let attrs = FieldAttrs::parse(&field.attrs, field.span())?;
        match attrs.kind {
            KindAttr::Set(_) => {
                if let Some(stages) = &attrs.stages {
                    return Err(syn::Error::new(
                        stages.span(),
                        "Unexpected stages for field with `#[set]` attribute",
                    ));
                }

                sets.push(Set {
                    field: field.clone(),
                });
            }
            KindAttr::PushConstants(Push { layout, .. }) => {
                let stages = attrs.stages.unwrap_or_else(|| Stages::new(field.span()));
                let layout = layout.unwrap_or_default();

                push_constants.push(PushConstants {
                    field: field.clone(),
                    stages,
                    layout,
                });
            }
        }
    }

    Ok(Input {
        item_struct,
        sets,
        push_constants,
    })
}
