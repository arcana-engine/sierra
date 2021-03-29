use {
    super::{layout::layout_type_name, parse::Input},
    proc_macro2::TokenStream,
};

pub(super) fn generate(input: &Input) -> TokenStream {
    let layout_ident = layout_type_name(input);

    let ident = &input.item_struct.ident;

    quote::quote!(
        impl #ident {
            pub fn layout(device: &::sierra::Device) -> Result<#layout_ident, ::sierra::OutOfMemory> {
                #layout_ident::new(device)
            }
        }

        impl ::sierra::PipelineInput for #ident {
            type Layout = #layout_ident;
            fn layout(device: &::sierra::Device) -> Result<#layout_ident, ::sierra::OutOfMemory> {

                Self::layout(device)
            }
        }
    )
}
