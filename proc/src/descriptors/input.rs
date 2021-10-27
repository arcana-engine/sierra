use {
    super::{instance::instance_type_name, layout::layout_type_name, parse::Input},
    proc_macro2::TokenStream,
};

pub(super) fn generate(input: &Input) -> TokenStream {
    std::iter::once(generate_input_impl(input))
        .chain(Some(generate_uniform_struct(input)))
        .collect::<TokenStream>()
}

fn generate_uniform_struct(input: &Input) -> TokenStream {
    let mut last_offset = quote::quote!(0);

    let fields: TokenStream = input.uniforms
        .iter()
        .map(|u| {
            let field_type = &u.field.ty;

            let val_ident = quote::format_ident!("val_{}", u.member);
            let pad_ident = quote::format_ident!("pad_{}", u.member);

            let field_align_mask = quote::quote!(<#field_type as ::sierra::ShaderRepr<::sierra::Std140>>::ALIGN_MASK);
            let pad_size = quote::quote!(::sierra::pad_size(#field_align_mask, #last_offset));
            let field_repr = quote::quote!(<#field_type as ::sierra::ShaderRepr<::sierra::Std140>>::Type);
            let next_offset = quote::quote!(::sierra::next_offset(#field_align_mask, #last_offset, ::std::mem::size_of::<#field_repr>()));

            // let offset = last_offset.clone();
            last_offset = next_offset;

            quote::quote! {
                pub #pad_ident: [u8; #pad_size],
                pub #val_ident: #field_repr,
            }
        })
        .collect();

    let update_fields: TokenStream = input.uniforms
        .iter()
        .map(|u| {
            let member = &u.member;
            let val_ident = quote::format_ident!("val_{}", u.member);

            quote::quote! {
                ::sierra::ShaderRepr::<::sierra::Std140>::copy_to_repr(&input.#member, &mut self.#val_ident);
            }
        })
        .collect();

    let align_mask = input.uniforms
            .iter()
            .fold(quote::quote!(15), |mut tokens, u| {
                let field_type = &u.field.ty;

                tokens.extend(
                    quote::quote! { | (<#field_type as ::sierra::ShaderRepr<::sierra::Std140>>::ALIGN_MASK) },
                );
                tokens
            });

    let pad_size = quote::quote!(::sierra::pad_size(#align_mask, #last_offset));

    let ident = &input.item_struct.ident;
    let uniforms_ident = quote::format_ident!("{}Uniforms", ident);
    let vis = &input.item_struct.vis;

    let doc_attr = if cfg!(feature = "verbose-docs") {
        format!(
            "#[doc = \"Combined uniforms for descriptors input [`{}`]\"]",
            ident
        )
        .parse()
        .unwrap()
    } else {
        quote::quote!(#[doc(hidden)])
    };

    quote::quote! {
        #[repr(C)]
        #[derive(Clone, Copy)]
        #doc_attr
        #vis struct #uniforms_ident {
            #fields
            pub end_pad: [u8; #pad_size],
        }

        unsafe impl ::sierra::Zeroable for #uniforms_ident {}
        unsafe impl ::sierra::Pod for #uniforms_ident {}

        impl #uniforms_ident {
            fn copy_from_input(&mut self, input: &#ident) {
                #update_fields
            }
        }
    }
}

fn generate_input_impl(input: &Input) -> TokenStream {
    let ident = &input.item_struct.ident;
    let layout_ident = layout_type_name(input);
    let instance_ident = instance_type_name(input);

    quote::quote! {
        impl ::sierra::DescriptorsInput for #ident {
            type Layout = #layout_ident;
            type Instance = #instance_ident;

            fn layout(device: &sierra::Device) -> ::std::result::Result<Self::Layout, ::sierra::OutOfMemory> {
                #layout_ident::new(device)
            }
        }
    }
}
