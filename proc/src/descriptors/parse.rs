use std::convert::TryFrom as _;

use syn::spanned::Spanned as _;

use crate::{
    find_unique_attribute,
    stage::{take_stages, Stage},
    take_attributes,
};

use super::{
    acceleration_structure::{parse_acceleration_structure_attr, AccelerationStructure},
    buffer::{parse_buffer_attr, Buffer},
    image::{parse_image_attr, Image},
    sampler::{parse_sampler_attr, Sampler},
    uniform::{parse_uniform_attr, Uniform},
    BindingFlag,
};

pub(super) struct Input {
    pub descriptors: Vec<Descriptor>,
    pub uniforms: Vec<UniformField>,
    pub item_struct: syn::ItemStruct,
}

pub struct Descriptor {
    pub stages: Vec<Stage>,
    pub flags: Vec<BindingFlag>,
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
    pub stages: Vec<Stage>,
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

pub(super) fn parse(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> syn::Result<Input> {
    if !attr.is_empty() {
        return Err(syn::Error::new_spanned(
            proc_macro2::TokenStream::from(attr),
            "#[descriptors] attribute does not accept arguments",
        ));
    }

    let mut item_struct = syn::parse::<syn::ItemStruct>(item)?;

    let mut uniforms = Vec::new();
    let mut descriptors = Vec::new();

    for (index, field) in item_struct.fields.iter_mut().enumerate() {
        let index = match u32::try_from(index) {
            Ok(index) => index,
            Err(_) => {
                return Err(syn::Error::new_spanned(field, "Too many fields"));
            }
        };

        match parse_input_field(field, index)? {
            None => {}
            Some(Field::Descriptor(descriptor)) => {
                descriptors.push(descriptor);
            }
            Some(Field::Uniform(uniform)) => uniforms.push(uniform),
        }
    }

    for descriptor in &descriptors {
        descriptor.validate(&item_struct)?;
    }

    for uniform in &uniforms {
        uniform.validate(&item_struct)?;
    }

    Ok(Input {
        item_struct,
        descriptors,
        uniforms,
    })
}

enum FieldAttribute {
    Sampler(Sampler),
    Image(Image),
    Buffer(Buffer),
    AccelerationStructure(AccelerationStructure),
    Uniform(Uniform),
}

enum Field {
    Uniform(UniformField),
    Descriptor(Descriptor),
}

fn parse_input_field(field: &mut syn::Field, field_index: u32) -> syn::Result<Option<Field>> {
    let ty = find_unique_attribute(
        &mut field.attrs,
        parse_input_field_attr,
        "At most one shader input type for field must be specified",
    )?;

    match ty {
        Some(ty) => {
            let stages = take_stages(&mut field.attrs)?;

            let flags: Vec<_> = if matches!(ty, FieldAttribute::Uniform(_)) {
                Vec::new()
            } else {
                take_attributes(&mut field.attrs, |attr| match attr.path.get_ident() {
                    Some(ident) if ident == "flags" => attr
                        .parse_args_with(|stream: syn::parse::ParseStream<'_>| {
                            let stages =
                                stream.parse_terminated::<_, syn::Token![,]>(BindingFlag::parse)?;
                            Ok(stages)
                        })
                        .map(Some),
                    _ => Ok(None),
                })?
                .into_iter()
                .flatten()
                .collect()
            };

            let member = match field.ident.as_ref() {
                None => syn::Member::Unnamed(syn::Index {
                    index: field_index,
                    span: field.span(),
                }),
                Some(field_ident) => syn::Member::Named(field_ident.clone()),
            };

            Ok(Some(match ty {
                FieldAttribute::Sampler(value) => Field::Descriptor(Descriptor {
                    desc_ty: DescriptorType::Sampler(value),
                    flags,
                    stages,
                    member,
                    field: field.clone(),
                }),
                FieldAttribute::Image(value) => Field::Descriptor(Descriptor {
                    desc_ty: DescriptorType::Image(value),
                    flags,
                    stages,
                    member,
                    field: field.clone(),
                }),
                FieldAttribute::Buffer(value) => Field::Descriptor(Descriptor {
                    desc_ty: DescriptorType::Buffer(value),
                    flags,
                    stages,
                    member,
                    field: field.clone(),
                }),
                FieldAttribute::AccelerationStructure(value) => Field::Descriptor(Descriptor {
                    desc_ty: DescriptorType::AccelerationStructure(value),
                    flags,
                    stages,
                    member,
                    field: field.clone(),
                }),
                FieldAttribute::Uniform(uniform) => Field::Uniform(UniformField {
                    field: field.clone(),
                    stages,
                    member,
                    uniform,
                }),
            }))
        }
        None => Ok(None),
    }
}

fn parse_input_field_attr(attr: &syn::Attribute) -> syn::Result<Option<FieldAttribute>> {
    on_first_ok!(parse_sampler_attr(attr)?.map(FieldAttribute::Sampler));
    on_first_ok!(parse_image_attr(attr)?.map(FieldAttribute::Image));
    on_first_ok!(parse_buffer_attr(attr)?.map(FieldAttribute::Buffer));
    on_first_ok!(
        parse_acceleration_structure_attr(attr)?.map(FieldAttribute::AccelerationStructure)
    );
    on_first_ok!(parse_uniform_attr(attr)?.map(FieldAttribute::Uniform));
    Ok(None)
}
