use crate::{get_unique, validate_member};

pub struct CombinedImageSampler {
    pub sampler: syn::Member,
}

impl CombinedImageSampler {
    pub fn validate(&self, item_struct: &syn::ItemStruct) -> syn::Result<()> {
        validate_member(&self.sampler, item_struct)?;
        Ok(())
    }
}

enum AttributeArgument {
    SeparateSampler { member: syn::Member },
}

pub(super) fn parse_combined_image_sampler_attr(
    attr: &syn::Attribute,
) -> syn::Result<Option<CombinedImageSampler>> {
    if attr
        .path
        .get_ident()
        .map_or(true, |i| i != "combined_image_sampler")
    {
        return Ok(None);
    }

    let args = attr.parse_args_with(|stream: syn::parse::ParseStream<'_>| {
        if stream.is_empty() {
            Ok(Vec::new())
        } else {
            let member = stream.parse::<syn::Member>()?;
            if !stream.is_empty() {
                Err(stream.error("Single member is expected in arguments"))
            } else {
                Ok(vec![AttributeArgument::SeparateSampler { member }])
            }
        }
    })?;

    let sampler = get_unique(
        args.iter().filter_map(|arg| match arg {
            AttributeArgument::SeparateSampler { member } => Some(member.clone()),
        }),
        attr,
        "Argument with sampler member must be specified",
    )?;

    Ok(Some(CombinedImageSampler { sampler }))
}
