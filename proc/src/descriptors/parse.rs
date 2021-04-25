use {
    super::{
        acceleration_structure::{parse_acceleration_structure_attr, AccelerationStructure},
        buffer::{parse_buffer_attr, Buffer},
        combined_image_sampler::{parse_combined_image_sampler_attr, CombinedImageSampler},
        sampled_image::{parse_sampled_image_attr, SampledImage},
        sampler::{parse_sampler_attr, Sampler},
        uniform::parse_uniform_attr,
    },
    crate::{find_unique_attribute, stage::Stage, take_attributes},
    std::convert::TryFrom as _,
    syn::spanned::Spanned as _,
};

pub struct Input {
    pub descriptors: Vec<Descriptor>,
    pub uniforms: Vec<Uniform>,
    pub item_struct: syn::ItemStruct,
}

pub struct Descriptor {
    pub stages: Vec<Stage>,
    pub desc_ty: DescriptorType,
    pub member: syn::Member,
    pub field: syn::Field,
}

impl Descriptor {
    fn validate(&self, item_struct: &syn::ItemStruct) -> syn::Result<()> {
        match &self.desc_ty {
            DescriptorType::Sampler(args) => args.validate(item_struct),
            DescriptorType::SampledImage(args) => args.validate(item_struct),
            DescriptorType::CombinedImageSampler(args) => args.validate(item_struct),
            DescriptorType::Buffer(args) => args.validate(item_struct),
            DescriptorType::AccelerationStructure(args) => args.validate(item_struct),
        }
    }
}

pub struct Uniform {
    pub stages: Vec<Stage>,
    pub field: syn::Field,
    pub member: syn::Member,
}

impl Uniform {
    #[inline]
    fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}

pub enum DescriptorType {
    Sampler(Sampler),
    SampledImage(SampledImage),
    CombinedImageSampler(CombinedImageSampler),
    Buffer(Buffer),
    AccelerationStructure(AccelerationStructure),
}

pub fn parse(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> syn::Result<Input> {
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
    SampledImage(SampledImage),
    CombinedImageSampler(CombinedImageSampler),
    Buffer(Buffer),
    AccelerationStructure(AccelerationStructure),
    Uniform,
}

enum Field {
    Uniform(Uniform),
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
            let stages: Vec<_> =
                take_attributes(&mut field.attrs, |attr| match attr.path.get_ident() {
                    Some(ident) if ident == "stages" => attr
                        .parse_args_with(|stream: syn::parse::ParseStream<'_>| {
                            let stages = stream.parse_terminated::<_, syn::Token![,]>(
                                |stream| match stream.parse::<syn::Ident>()? {
                                    i if i == "Vertex" => Ok(Stage::Vertex),
                                    i if i == "TessellationControl" => {
                                        Ok(Stage::TessellationControl)
                                    }
                                    i if i == "TessellationEvaluation" => {
                                        Ok(Stage::TessellationEvaluation)
                                    }
                                    i if i == "Geometry" => Ok(Stage::Geometry),
                                    i if i == "Fragment" => Ok(Stage::Fragment),
                                    i if i == "Compute" => Ok(Stage::Compute),
                                    i if i == "Raygen" => Ok(Stage::Raygen),
                                    i if i == "AnyHit" => Ok(Stage::AnyHit),
                                    i if i == "ClosestHit" => Ok(Stage::ClosestHit),
                                    i if i == "Miss" => Ok(Stage::Miss),
                                    i if i == "Intersection" => Ok(Stage::Intersection),
                                    i => Err(stream.error(format!("Unrecognized stage `{}`", i))),
                                },
                            )?;
                            Ok(stages)
                        })
                        .map(Some),
                    _ => Ok(None),
                })?
                .into_iter()
                .flatten()
                .collect();

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
                    stages,
                    member,
                    field: field.clone(),
                }),
                FieldAttribute::SampledImage(value) => Field::Descriptor(Descriptor {
                    desc_ty: DescriptorType::SampledImage(value),
                    stages,
                    member,
                    field: field.clone(),
                }),
                FieldAttribute::CombinedImageSampler(value) => Field::Descriptor(Descriptor {
                    desc_ty: DescriptorType::CombinedImageSampler(value),
                    stages,
                    member,
                    field: field.clone(),
                }),
                FieldAttribute::Buffer(value) => Field::Descriptor(Descriptor {
                    desc_ty: DescriptorType::Buffer(value),
                    stages,
                    member,
                    field: field.clone(),
                }),
                FieldAttribute::AccelerationStructure(value) => Field::Descriptor(Descriptor {
                    desc_ty: DescriptorType::AccelerationStructure(value),
                    stages,
                    member,
                    field: field.clone(),
                }),
                FieldAttribute::Uniform => Field::Uniform(Uniform {
                    field: field.clone(),
                    stages,
                    member,
                }),
            }))
        }
        None => Ok(None),
    }
}

fn parse_input_field_attr(attr: &syn::Attribute) -> syn::Result<Option<FieldAttribute>> {
    on_first_ok!(parse_sampler_attr(attr)?.map(FieldAttribute::Sampler));
    on_first_ok!(parse_sampled_image_attr(attr)?.map(FieldAttribute::SampledImage));
    on_first_ok!(parse_combined_image_sampler_attr(attr)?.map(FieldAttribute::CombinedImageSampler));
    on_first_ok!(parse_buffer_attr(attr)?.map(FieldAttribute::Buffer));
    on_first_ok!(
        parse_acceleration_structure_attr(attr)?.map(FieldAttribute::AccelerationStructure)
    );
    on_first_ok!(parse_uniform_attr(attr)?.map(|_| FieldAttribute::Uniform));
    Ok(None)
}
