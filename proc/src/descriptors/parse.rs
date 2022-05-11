use std::convert::TryFrom;

use proc_easy::EasyAttributes;
use syn::spanned::Spanned;

use crate::{flags::BindingFlags, kw, stage::Stages};

use super::{
    acceleration_structure::AccelerationStructure, buffer::Buffer, image::Image, sampler::Sampler,
    uniform::Uniform,
};

pub(super) struct Input {
    pub descriptors: Vec<Descriptor>,
    pub uniforms: Vec<UniformField>,
    pub item_struct: syn::ItemStruct,
    pub cycle_capacity: usize,
}

pub struct Descriptor {
    pub stages: Stages,
    pub flags: BindingFlags,
    pub desc_ty: DescriptorType,
    pub member: syn::Member,
    pub field: syn::Field,
}

impl Descriptor {
    #[inline]
    fn validate(&self, item_struct: &syn::ItemStruct) -> syn::Result<()> {
        match &self.desc_ty {
            DescriptorType::Sampler(args) => args.validate(item_struct),
            DescriptorType::Image(args) => args.validate(item_struct),
            DescriptorType::Buffer(args) => args.validate(item_struct),
            DescriptorType::AccelerationStructure(args) => args.validate(item_struct),
        }
    }
}

pub(super) struct UniformField {
    pub stages: Stages,
    pub field: syn::Field,
    pub member: syn::Member,
    pub uniform: Uniform,
}

impl UniformField {
    #[inline]
    fn validate(&self, item_struct: &syn::ItemStruct) -> syn::Result<()> {
        self.uniform.validate(item_struct)
    }
}

pub enum DescriptorType {
    Sampler(Sampler),
    Image(Image),
    Buffer(Buffer),
    AccelerationStructure(AccelerationStructure),
}

proc_easy::easy_argument_value! {
    struct Capacity {
        kw: kw::capacity,
        lit: syn::LitInt,
    }
}

proc_easy::easy_attributes! {
    @(sierra)
    struct DescriptorsAttributes {
        capacity: Option<Capacity>,
    }
}

proc_easy::easy_argument_group! {
    #[derive(Clone)]
    enum Kind {
        AccelerationStructure(AccelerationStructure),
        Buffer(Buffer),
        Image(Image),
        Sampler(Sampler),
        Uniform(Uniform),
    }
}

proc_easy::easy_attributes! {
    @(sierra)
    struct FieldAttributes {
        kind: Kind,
        stages: Stages,
        flags: Option<BindingFlags>,
    }
}

pub(super) fn parse(item: proc_macro::TokenStream) -> syn::Result<Input> {
    let mut item_struct = syn::parse::<syn::ItemStruct>(item)?;

    let attrs = DescriptorsAttributes::parse(&item_struct.attrs, item_struct.ident.span())?;
    let cycle_capacity = match &attrs.capacity {
        None => 5,
        Some(capacity) => capacity.lit.base10_parse()?,
    };

    let mut uniforms = Vec::new();
    let mut descriptors = Vec::new();

    for (index, field) in item_struct.fields.iter_mut().enumerate() {
        let index = match u32::try_from(index) {
            Ok(index) => index,
            Err(_) => {
                return Err(syn::Error::new_spanned(field, "Too many fields"));
            }
        };

        let attrs = FieldAttributes::parse(&field.attrs, field.span())?;

        let member = match &field.ident {
            None => syn::Member::Unnamed(syn::Index {
                span: field.span(),
                index,
            }),
            Some(ident) => syn::Member::Named(ident.clone()),
        };

        match attrs.kind {
            Kind::Sampler(value) => descriptors.push(Descriptor {
                desc_ty: DescriptorType::Sampler(value),
                flags: attrs.flags.unwrap_or_default(),
                stages: attrs.stages,
                member,
                field: field.clone(),
            }),
            Kind::Image(value) => descriptors.push(Descriptor {
                desc_ty: DescriptorType::Image(value),
                flags: attrs.flags.unwrap_or_default(),
                stages: attrs.stages,
                member,
                field: field.clone(),
            }),
            Kind::Buffer(value) => descriptors.push(Descriptor {
                desc_ty: DescriptorType::Buffer(value),
                flags: attrs.flags.unwrap_or_default(),
                stages: attrs.stages,
                member,
                field: field.clone(),
            }),
            Kind::AccelerationStructure(value) => descriptors.push(Descriptor {
                desc_ty: DescriptorType::AccelerationStructure(value),
                flags: attrs.flags.unwrap_or_default(),
                stages: attrs.stages,
                member,
                field: field.clone(),
            }),
            Kind::Uniform(uniform) => {
                if let Some(flags) = &attrs.flags {
                    return Err(syn::Error::new(
                        flags.span(),
                        "Unexpected binding flags on uniform field",
                    ));
                }

                uniforms.push(UniformField {
                    field: field.clone(),
                    stages: attrs.stages,
                    member,
                    uniform,
                })
            }
        }
    }

    for descriptor in &descriptors {
        descriptor.validate(&item_struct)?;
    }

    for uniform in &uniforms {
        uniform.validate(&item_struct)?;
    }

    Ok(Input {
        cycle_capacity,
        item_struct,
        descriptors,
        uniforms,
    })
}
