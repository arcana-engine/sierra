use crate::find_unique;

pub struct CombinedImageSampler {
    pub separate_sampler: Option<syn::Member>,
}

enum AttributeArgument {
    SeparateSampler { member: syn::Member },
}

pub(super) fn parse_combined_image_sampler_attr(
    attr: &syn::Attribute,
) -> Option<CombinedImageSampler> {
    if attr
        .path
        .get_ident()
        .map_or(true, |i| i != "combined_image_sampler")
    {
        return None;
    }

    let args = attr
        .parse_args_with(|stream: syn::parse::ParseStream<'_>| {
            Ok(if stream.is_empty() {
                Vec::new()
            } else {
                let member = stream.parse::<syn::Member>()?;
                vec![AttributeArgument::SeparateSampler { member }]
            })
        })
        .unwrap();

    let separate_sampler = find_unique(
        args.iter().filter_map(|arg| match arg {
            AttributeArgument::SeparateSampler { member } => Some(member.clone()),
        }),
        "Expected at most one `sampler` argument",
    );

    Some(CombinedImageSampler { separate_sampler })
}
