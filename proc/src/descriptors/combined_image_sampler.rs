use crate::{find_unique, validate_member};

pub struct CombinedImageSampler {
    pub separate_sampler: Option<syn::Member>,
}

impl CombinedImageSampler {
    pub fn validate(&self, item_struct: &syn::ItemStruct) -> syn::Result<()> {
        match &self.separate_sampler {
            None => Ok(()),
            Some(member) => validate_member(member, item_struct),
        }
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

    let separate_sampler = find_unique(
        args.iter().filter_map(|arg| match arg {
            AttributeArgument::SeparateSampler { member } => Some(member.clone()),
        }),
        attr,
        "Expected at most one `sampler` argument",
    )?;

    Ok(Some(CombinedImageSampler { separate_sampler }))
}
