use proc_macro2::TokenStream;

use crate::layout::StructLayout;

use super::parse::Input;

fn generate_layout(item_struct: &syn::ItemStruct, layout: StructLayout) -> TokenStream {
    let vis = &item_struct.vis;

    let sierra_layout = layout.sierra_type();
    let layout_name = layout.name();

    let mut offset = quote::quote!(0);

    let fields: TokenStream = item_struct
        .fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let field_type = &field.ty;

            let val_ident = quote::format_ident!("val_{}", ident);
            // let off_ident = quote::format_ident!("off_{}", ident);
            let pad_ident = quote::format_ident!("pad_{}", ident);

            let field_align_mask = quote::quote!(<#field_type as ::sierra::ShaderRepr<#sierra_layout>>::ALIGN_MASK);
            let pad_size = quote::quote!(::sierra::pad_size(#field_align_mask, #offset));
            let field_repr = quote::quote!(<#field_type as ::sierra::ShaderRepr<#sierra_layout>>::Type);
            offset = quote::quote!(::sierra::next_offset(#field_align_mask, #offset, ::std::mem::size_of::<#field_repr>()));

            quote::quote! {
                pub #pad_ident: [u8; #pad_size],
                // pub #off_ident: [(); #offset],
                pub #val_ident: #field_repr,
            }
        })
        .collect();

    let update_fields: TokenStream = item_struct
        .fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let val_ident = quote::format_ident!("val_{}", ident);

            quote::quote! {
                ::sierra::ShaderRepr::<#sierra_layout>::copy_to_repr(&self.#ident, &mut repr.#val_ident);
            }
        })
        .collect();

    let align_mask = item_struct
        .fields
        .iter()
        .fold(quote::quote!(15), |mut tokens, field| {
            let field_type = &field.ty;

            tokens.extend(
                quote::quote! { | (<#field_type as ::sierra::ShaderRepr<#sierra_layout>>::ALIGN_MASK) },
            );
            tokens
        });

    let pad_size = quote::quote!(::sierra::pad_size(#align_mask, #offset));

    let ident = &item_struct.ident;
    let std_ident = quote::format_ident!("{}Repr{}", item_struct.ident, layout_name);

    let doc_attr = if cfg!(feature = "verbose-docs") {
        format!(
            "#[doc = \"Structure generated to represent [`{}`] in shader with {} compatible layout\"]" ,
            ident,
            layout_name,
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
        #vis struct #std_ident {
            #fields
            pub end_pad: [u8; #pad_size],
        }

        unsafe impl ::sierra::bytemuck::Zeroable for #std_ident {}
        unsafe impl ::sierra::bytemuck::Pod for #std_ident {}

        impl ::sierra::ShaderRepr<#sierra_layout> for #ident {
            const ALIGN_MASK: usize = #align_mask;
            const ARRAY_PADDING: usize = 0;
            type Type = #std_ident;
            type ArrayPadding = [u8; 0];
            fn copy_to_repr(&self, repr: &mut #std_ident) {
                #update_fields
            }
        }
    }
}

pub(super) fn generate_repr(input: &Input) -> TokenStream {
    let mut output = TokenStream::new();

    if input.layouts.flags.is_empty() {
        output.extend(generate_layout(
            &input.item_struct,
            StructLayout::Std140(Default::default()),
        ));
        output.extend(generate_layout(
            &input.item_struct,
            StructLayout::Std430(Default::default()),
        ));
    } else {
        for layout in &input.layouts.flags {
            output.extend(generate_layout(&input.item_struct, *layout));
        }
    }
    output
}
