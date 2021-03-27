use crate::find_unique;

pub struct CombinedImageSampler {
    pub separate_sampler: Option<syn::Member>,
}

enum AttributeArgument {
    SeparateSampler { member: syn::Member },
}

pub(crate) fn parse_combined_image_sampler_attr(
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
                stream
                    .parse_terminated::<_, syn::Token![,]>(|stream| {
                        let ident = stream.parse::<syn::Ident>()?;

                        match ident {
                            ident if ident == "sampler" => {
                                if !stream.is_empty() {
                                    stream.parse::<syn::Token![=]>()?;
                                    let member = stream.parse::<syn::Member>()?;
                                    Ok(AttributeArgument::SeparateSampler { member })
                                } else {
                                    Ok(AttributeArgument::SeparateSampler {
                                        member: syn::Member::Named(quote::format_ident!("sampler")),
                                    })
                                }
                            }
                            _ => {
                                return Err(stream.error("Unrecognized argument"));
                            }
                        }
                    })?
                    .into_iter()
                    .collect()
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
