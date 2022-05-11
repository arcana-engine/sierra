use crate::kw;

impl Buffer {
    #[inline]
    pub fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}

proc_easy::easy_argument_group! {
    #[derive(Clone, Copy)]
    pub enum Kind {
        Uniform(kw::uniform),
        Storage(kw::storage),
    }
}

proc_easy::easy_parse! {
    #[derive(Clone, Copy)]
    pub enum BufferArgs {
        Kind(Kind),
        Texel(kw::texel),
    }
}

proc_easy::easy_argument_tuple! {
    #[derive(Clone)]
    pub struct Buffer {
        pub kw: kw::buffer,
        pub kind: Option<Kind>,
        pub texel: Option<kw::texel>,
    }
}
