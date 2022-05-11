use crate::{kw, layout::StructLayout};

impl Uniform {
    #[inline]
    pub fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}

proc_easy::easy_argument_tuple! {
    #[derive(Clone, Copy)]
    pub struct Uniform {
        pub kw: kw::uniform,
        pub layout: Option<StructLayout>,
    }
}
