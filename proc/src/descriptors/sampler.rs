use crate::kw;

proc_easy::easy_argument! {
    #[derive(Clone, Copy)]
    pub struct Sampler {
        pub kw: kw::sampler,
    }
}

impl Sampler {
    #[inline]
    pub fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}
