use {
    super::{layout::layout_type_name, parse::Input},
    proc_macro2::TokenStream,
};

pub(super) fn instance_type_name(input: &Input) -> syn::Ident {
    quote::format_ident!("{}Instance", input.item_struct.ident)
}

pub(super) fn generate(input: &Input) -> TokenStream {
    let layout_ident = layout_type_name(input);
    let instance_ident = instance_type_name(input);

    let instance_sets = input
        .sets
        .iter()
        .map(|set| {
            let ident = &set.ident;
            let ty = &set.ty;
            quote::quote!(
                pub #ident: &'a <#ty as ::sierra::DescriptorsInput>::Instance,
            )
        })
        .collect::<TokenStream>();

    let sets_get_updated = input
        .sets
        .iter()
        .map(|set| {
            let ident = &set.ident;
            quote::quote!(
                std::clone::Clone::clone(::sierra::DescriptorsInstance::get_updated(self.#ident, fence))
            )
        })
        .collect::<Vec<_>>();

    let vis = &input.item_struct.vis;
    let ident = &input.item_struct.ident;

    let doc_attr = if cfg!(feature = "verbose-docs") {
        format!(
            "#[doc = \"[`sierra::TypedPipelineLayout`] implementation for [`{}`]\"]",
            ident
        )
        .parse()
        .unwrap()
    } else {
        quote::quote!(#[doc(hidden)])
    };

    quote::quote!(
        #doc_attr
        #vis struct #instance_ident<'a> {
            pub layout: &'a #layout_ident,
            #instance_sets
        }

        impl<'a> #instance_ident<'a> {
            pub fn bind_graphics(&self, fence: usize, bump: &'a ::sierra::bumpalo::Bump, encoder: &mut ::sierra::EncoderCommon<'a>) {

                let sets = &*bump.alloc([
                    #(#sets_get_updated),*
                ]);

                encoder.bind_graphics_descriptor_sets(
                    #layout_ident::raw(self.layout),
                    0,
                    sets,
                    &[],
                )
            }
        }
    )
    .into()
}
