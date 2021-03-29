use {super::parse::Input, proc_macro2::TokenStream};

pub(super) fn layout_type_name(input: &Input) -> syn::Ident {
    quote::format_ident!("{}Layout", input.item_struct.ident)
}

pub(super) fn generate(input: &Input) -> TokenStream {
    let layout_ident = layout_type_name(input);

    let layout_sets = input
        .sets
        .iter()
        .map(|set| {
            let ident = &set.ident;
            let ty = &set.ty;
            quote::quote!(
                pub #ident: <#ty as ::sierra::DescriptorsInput>::Layout,
            )
        })
        .collect::<TokenStream>();

    let layout_sets_new = input
        .sets
        .iter()
        .map(|set| {
            let ident = &set.ident;
            let ty = &set.ty;
            quote::quote!(
                let #ident = <#ty as ::sierra::DescriptorsInput>::layout(device)?;
            )
        })
        .collect::<TokenStream>();

    let layout_sets_init = input
        .sets
        .iter()
        .map(|set| {
            let ident = &set.ident;
            quote::quote!(
                #ident,
            )
        })
        .collect::<TokenStream>();

    let raw_set_layouts = input
        .sets
        .iter()
        .map(|set| {
            let ident = &set.ident;
            quote::quote!(
                ::std::clone::Clone::clone(::sierra::DescriptorsLayout::raw(&#ident))
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
        #[derive(Clone, Debug)]
        #doc_attr
        #vis struct #layout_ident {
            pub pipeline_layout: ::sierra::PipelineLayout,
            #layout_sets
        }

        impl #layout_ident {
            pub fn new(device: &::sierra::Device) -> ::std::result::Result<Self, ::sierra::OutOfMemory> {
                #layout_sets_new

                let pipeline_layout = device.create_pipeline_layout(::sierra::PipelineLayoutInfo {
                    sets: ::std::vec![#(#raw_set_layouts),*],
                    push_constants: ::std::vec::Vec::new(),
                })?;

                Ok(#layout_ident {
                    pipeline_layout,
                    #layout_sets_init
                })
            }

            pub fn raw(&self) -> &::sierra::PipelineLayout {
                &self.pipeline_layout
            }
        }

        impl ::sierra::TypedPipelineLayout for #layout_ident {
            fn new(device: &::sierra::Device) -> ::std::result::Result<Self, ::sierra::OutOfMemory> {
                Self::new(device)
            }

            fn raw(&self) -> &::sierra::PipelineLayout {
                self.raw()
            }
        }

    )
    .into()
}
