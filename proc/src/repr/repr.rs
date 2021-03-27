use {super::parse::Input, proc_macro2::TokenStream};

pub fn generate_repr(input: &Input) -> TokenStream {
    let vis = &input.item_struct.vis;

    let mut last_offset = quote::quote!(0);

    let fields_140: TokenStream = input
        .fields
        .iter()
        .flat_map(|field| {
            let field_type = field.as_type.as_ref().unwrap_or(&field.ty);

            let val_ident = quote::format_ident!("val_{}", field.ident);
            // let off_ident = quote::format_ident!("off_{}", field.ident);
            let pad_ident = quote::format_ident!("pad_{}", field.ident);

            let field_align_mask = quote::quote!(<#field_type as ::sierra::ShaderRepr<::sierra::Std140>>::ALIGN_MASK);
            let pad_size = quote::quote!(::sierra::pad_size(#field_align_mask, #last_offset));
            let field_repr = quote::quote!(<#field_type as ::sierra::ShaderRepr<::sierra::Std140>>::Type);
            let next_offset = quote::quote!(::sierra::next_offset(#field_align_mask, #last_offset, ::core::mem::size_of::<#field_repr>()));

            // let offset = last_offset.clone();
            last_offset = next_offset;

            quote::quote! {
                pub #pad_ident: [u8; #pad_size],
                // pub #off_ident: [(); #offset],
                pub #val_ident: #field_repr,
            }
        })
        .collect();

    let fields_430: TokenStream = input
        .fields
        .iter()
        .flat_map(|field| {
            let field_type = field.as_type.as_ref().unwrap_or(&field.ty);

            let val_ident = quote::format_ident!("val_{}", field.ident);
            // let off_ident = quote::format_ident!("off_{}", field.ident);
            let pad_ident = quote::format_ident!("pad_{}", field.ident);

            let field_align_mask = quote::quote!(<#field_type as ::sierra::ShaderRepr<::sierra::Std430>>::ALIGN_MASK);
            let pad_size = quote::quote!(::sierra::pad_size(#field_align_mask, #last_offset));
            let field_repr = quote::quote!(<#field_type as ::sierra::ShaderRepr<::sierra::Std430>>::Type);
            let next_offset = quote::quote!(::sierra::next_offset(#field_align_mask, #last_offset, ::core::mem::size_of::<#field_repr>()));

            // let offset = last_offset.clone();
            last_offset = next_offset;

            quote::quote! {
                pub #pad_ident: [u8; #pad_size],
                // pub #off_ident: [(); #offset],
                pub #val_ident: #field_repr,
            }
        })
        .collect();

    let from_fields_140: TokenStream = input
        .fields
        .iter()
        .flat_map(|field| {
            let field_ident = &field.ident;
            let val_ident = quote::format_ident!("val_{}", field.ident);
            let pad_ident = quote::format_ident!("pad_{}", field.ident);

            match &field.as_type {
                Some(field_as_type) => {
                    quote::quote! {
                        #val_ident: ::sierra::ShaderRepr::<::sierra::Std140>::copy_to_repr(&#field_as_type::from(value.#field_ident)),
                        #pad_ident: Default::default(),
                    }
                }
                None => {
                    quote::quote! {
                        #val_ident: ::sierra::ShaderRepr::<::sierra::Std140>::copy_to_repr(&value.#field_ident),
                        #pad_ident: Default::default(),
                    }
                }
            }
        })
        .collect();

    let from_fields_430: TokenStream = input
        .fields
        .iter()
        .flat_map(|field| {
            let field_ident = &field.ident;
            let val_ident = quote::format_ident!("val_{}", field.ident);
            let pad_ident = quote::format_ident!("pad_{}", field.ident);

            match &field.as_type {
                Some(field_as_type) => {
                    quote::quote! {
                        #val_ident: ::sierra::ShaderRepr::<::sierra::Std430>::copy_to_repr(&#field_as_type::from(value.#field_ident)),
                        #pad_ident: Default::default(),
                    }
                }
                None => {
                    quote::quote! {
                        #val_ident: ::sierra::ShaderRepr::<::sierra::Std430>::copy_to_repr(&value.#field_ident),
                        #pad_ident: Default::default(),
                    }
                }
            }
        })
        .collect();

    let align_mask_140 = input
        .fields
        .iter()
        .fold(quote::quote!(15), |mut tokens, field| {
            let field_type = field.as_type.as_ref().unwrap_or(&field.ty);

            tokens.extend(
                quote::quote! { | (<#field_type as ::sierra::ShaderRepr<::sierra::Std140>>::ALIGN_MASK) },
            );
            tokens
        });

    let align_mask_430 = input
        .fields
        .iter()
        .fold(quote::quote!(0), |mut tokens, field| {
            let field_type = field.as_type.as_ref().unwrap_or(&field.ty);

            tokens.extend(
                quote::quote! { | (<#field_type as ::sierra::ShaderRepr<::sierra::Std430>>::ALIGN_MASK) },
            );
            tokens
        });

    let pad_size_140 = quote::quote!(::sierra::pad_size(#align_mask_140, #last_offset));
    let pad_size_430 = quote::quote!(::sierra::pad_size(#align_mask_430, #last_offset));

    let ident = &input.item_struct.ident;
    let std140_ident = quote::format_ident!("{}ReprStd140", input.item_struct.ident);
    let std430_ident = quote::format_ident!("{}ReprStd430", input.item_struct.ident);

    quote::quote! {
        #[repr(C)]
        #[derive(Clone, Copy)]
        #vis struct #std140_ident {
            #fields_140
            pub end_pad: [u8; #pad_size_140],
        }

        unsafe impl ::sierra::Zeroable for #std140_ident {}
        unsafe impl ::sierra::Pod for #std140_ident {}

        impl #std140_ident {
            fn copy_from_rust(value: &#ident) -> #std140_ident {
                #std140_ident {
                    #from_fields_140
                    end_pad: Default::default(),
                }
            }
        }

        impl ::sierra::ShaderRepr<::sierra::Std140> for #ident {
            const ALIGN_MASK: usize = #align_mask_140;
            const ARRAY_PADDING: usize = 0;
            type Type = #std140_ident;
            type ArrayPadding = [u8; 0];
            fn copy_to_repr(&self) -> #std140_ident {
                #std140_ident::copy_from_rust(self)
            }
        }

        #[repr(C)]
        #[derive(Clone, Copy)]
        #vis struct #std430_ident {
            #fields_430
            pub end_pad: [u8; #pad_size_430],
        }

        unsafe impl ::sierra::Zeroable for #std430_ident {}
        unsafe impl ::sierra::Pod for #std430_ident {}

        impl #std430_ident {
            fn copy_from_rust(value: &#ident) -> #std430_ident {
                #std430_ident {
                    #from_fields_430
                    end_pad: Default::default(),
                }
            }
        }

        impl ::sierra::ShaderRepr<::sierra::Std430> for #ident {
            const ALIGN_MASK: usize = #align_mask_430;
            const ARRAY_PADDING: usize = 0;
            type Type = #std430_ident;
            type ArrayPadding = [u8; 0];
            fn copy_to_repr(&self) -> #std430_ident {
                #std430_ident::copy_from_rust(self)
            }
        }

    }
}
