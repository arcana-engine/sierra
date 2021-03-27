use {
    super::{
        acceleration_structure::{parse_acceleration_structure_attr, AccelerationStructure},
        buffer::{parse_buffer_attr, Buffer},
        combined_image_sampler::{parse_combined_image_sampler_attr, CombinedImageSampler},
    },
    crate::{find_unique, stage::Stage},
    std::convert::TryFrom as _,
    syn::spanned::Spanned as _,
};

pub struct Input {
    pub fields: Vec<Field>,
    pub item_struct: syn::ItemStruct,
}

pub struct Field {
    pub stages: Vec<Stage>,
    pub ty: FieldType,
    pub binding: u32,
    pub member: syn::Member,
}

pub enum FieldType {
    CombinedImageSampler(CombinedImageSampler),
    AccelerationStructure(AccelerationStructure),
    Buffer(Buffer),
}

impl FieldType {
    pub fn is_descriptor(&self) -> bool {
        match self {
            Self::CombinedImageSampler(_) => true,
            Self::AccelerationStructure(_) => true,
            Self::Buffer(_) => true,
        }
    }
}

pub fn parse(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> Input {
    assert!(attr.is_empty());

    let mut item_struct = syn::parse::<syn::ItemStruct>(item)
        .expect("`#[pipeline_layout]` can be applied only to structs");

    let mut binding = 0;

    let fields = item_struct
        .fields
        .iter_mut()
        .enumerate()
        .filter_map(|(index, field)| {
            parse_field_attrs(field, u32::try_from(index).unwrap(), &mut binding)
        })
        .collect::<Vec<_>>();

    Input {
        item_struct,
        fields,
    }
}

fn parse_field_attrs(field: &mut syn::Field, field_index: u32, binding: &mut u32) -> Option<Field> {
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

    *binding += 1;
    Some(Field {
        ty,
        stages,
        binding: *binding - 1,
        member,
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
        Ok(())
    }

    inner(attr).err()
}
