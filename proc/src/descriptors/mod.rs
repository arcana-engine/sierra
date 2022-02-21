mod acceleration_structure;
mod buffer;
// mod combined_image_sampler;
mod image;
mod input;
mod instance;
mod layout;
mod parse;
mod sampler;
mod uniform;

use {proc_macro2::TokenStream, quote::TokenStreamExt as _, std::collections::HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingFlag {
    UpdateAfterBind,
    PartiallyBound,
    UpdateUnused,
}

fn binding_flag_tokens(stage: BindingFlag) -> TokenStream {
    match stage {
        BindingFlag::UpdateAfterBind => {
            quote::quote!(::sierra::DescriptorBindingFlags::UPDATE_AFTER_BIND)
        }
        BindingFlag::PartiallyBound => {
            quote::quote!(::sierra::DescriptorBindingFlags::PARTIALLY_BOUND)
        }
        BindingFlag::UpdateUnused => {
            quote::quote!(::sierra::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING)
        }
    }
}

fn combined_binding_flags(flags: impl IntoIterator<Item = BindingFlag>) -> TokenStream {
    let mut flags = flags.into_iter();
    if let Some(head) = flags.next() {
        let or = proc_macro2::Punct::new('|', proc_macro2::Spacing::Alone);
        let mut stream = binding_flag_tokens(head);
        for stage in flags {
            stream.append(or.clone());
            stream.extend(binding_flag_tokens(stage));
        }
        stream
    } else {
        quote::quote!(::sierra::DescriptorBindingFlags::empty())
    }
}

fn combined_binding_flags_dedup(flags: impl IntoIterator<Item = BindingFlag>) -> TokenStream {
    let flags = flags.into_iter().collect::<HashSet<_>>();
    combined_binding_flags(flags)
}

pub fn descriptors(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro2::TokenStream {
    match parse::parse(attr, item) {
        Ok(input) => {
            let item_struct = &input.item_struct;
            std::iter::once(quote::quote!(#item_struct))
                .chain(Some(input::generate(&input)))
                .chain(Some(instance::generate(&input)))
                .chain(Some(layout::generate(&input)))
                // .chain(Some(generate_glsl_shader_input(&input)))
                .collect::<proc_macro2::TokenStream>()
        }
        Err(err) => err.into_compile_error(),
    }
}
