use {
    super::{
        acceleration_structure::{parse_acceleration_structure_attr, AccelerationStructure},
        buffer::{parse_buffer_attr, Buffer},
        combined_image_sampler::{parse_combined_image_sampler_attr, CombinedImageSampler},
        uniform::parse_uniform_attr,
    },
    crate::{find_unique, stage::Stage},
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
    pub ty: DescriptorType,
    pub member: syn::Member,
}

pub struct Uniform {
    pub stages: Vec<Stage>,
    pub ty: syn::Type,
    pub member: syn::Member,
}

pub enum DescriptorType {
    CombinedImageSampler(CombinedImageSampler),
    AccelerationStructure(AccelerationStructure),
    Buffer(Buffer),
}

pub fn parse(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> Input {
    assert!(attr.is_empty());

    let mut item_struct = syn::parse::<syn::ItemStruct>(item)
        .expect("`#[pipeline_layout]` can be applied only to structs");

    let mut uniforms = Vec::new();
    let mut descriptors = Vec::new();

    for (index, field) in item_struct.fields.iter_mut().enumerate() {
        match parse_field_attrs(field, u32::try_from(index).unwrap()) {
            None => {}
            Some(Field::Descriptor(descriptor)) => descriptors.push(descriptor),
            Some(Field::Uniform(uniform)) => uniforms.push(uniform),
        }
    }

    Input {
        item_struct,
        descriptors,
        uniforms,
    }
}

enum FieldType {
    CombinedImageSampler(CombinedImageSampler),
    AccelerationStructure(AccelerationStructure),
    Buffer(Buffer),
    Uniform,
}

enum Field {
    Uniform(Uniform),
    Descriptor(Descriptor),
}

fn parse_field_attrs(field: &mut syn::Field, field_index: u32) -> Option<Field> {
    let (ty, index) = find_unique(
        field.attrs.iter().enumerate().filter_map(|(index, attr)| {
            let ty = parse_input_field_type(attr)?;
            Some((ty, index))
        }),
        "At most one shader input type for field must be specified",
    )?;
    field.attrs.swap_remove(index);

    let stages = find_unique(
        field.attrs.iter().enumerate().filter_map(|(index, attr)| {
            let ident = attr.path.get_ident()?;
            if ident != "stages" {
                None
            } else {
                Some((
                    index,
                    attr.parse_args_with(|stream: syn::parse::ParseStream<'_>| {
                        let stages = stream
                            .parse_terminated::<_, syn::Token![,]>(|stream| {
                                match stream.parse::<syn::Ident>()? {
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
                                }
                            })?
                            .into_iter()
                            .collect::<Vec<_>>();
                        Ok(stages)
                    })
                    .expect("Failed to parse `stages` arg"),
                ))
            }
        }),
        "Expected at most one `stages` attribute",
    );

    let stages = match stages {
        Some((index, stages)) => {
            field.attrs.swap_remove(index);
            stages
        }
        _ => {
            vec![]
        }
    };

    let member = match field.ident.as_ref() {
        None => syn::Member::Unnamed(syn::Index {
            index: field_index,
            span: field.span(),
        }),
        Some(field_ident) => syn::Member::Named(field_ident.clone()),
    };

    Some(match ty {
        FieldType::Uniform => Field::Uniform(Uniform {
            ty: field.ty.clone(),
            stages,
            member,
        }),
        FieldType::CombinedImageSampler(value) => Field::Descriptor(Descriptor {
            ty: DescriptorType::CombinedImageSampler(value),
            stages,
            member,
        }),
        FieldType::AccelerationStructure(value) => Field::Descriptor(Descriptor {
            ty: DescriptorType::AccelerationStructure(value),
            stages,
            member,
        }),
        FieldType::Buffer(value) => Field::Descriptor(Descriptor {
            ty: DescriptorType::Buffer(value),
            stages,
            member,
        }),
    })
}

fn parse_input_field_type(attr: &syn::Attribute) -> Option<FieldType> {
    fn some_err<T>(opt: Option<T>) -> Result<(), T> {
        opt.map(Err).unwrap_or(Ok(()))
    }

    fn inner(attr: &syn::Attribute) -> Result<(), FieldType> {
        some_err(parse_combined_image_sampler_attr(attr).map(FieldType::CombinedImageSampler))?;
        some_err(parse_acceleration_structure_attr(attr).map(FieldType::AccelerationStructure))?;
        some_err(parse_buffer_attr(attr).map(FieldType::Buffer))?;
        some_err(parse_uniform_attr(attr).map(|_| FieldType::Uniform))?;
        Ok(())
    }

    inner(attr).err()
}
