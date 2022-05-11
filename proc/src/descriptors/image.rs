use crate::kw;

impl Image {
    #[inline]
    pub fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}

proc_easy::easy_argument_group! {
    #[derive(Clone, Copy)]
    pub enum Kind {
        Sampled(kw::sampled),
        Storage(kw::storage),
    }
}

proc_easy::easy_argument_tuple! {
    #[derive(Clone, Copy)]
    pub struct Image {
        pub kw: kw::image,
        pub kind: Option<Kind>,
    }
}
