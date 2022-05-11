use crate::kw;

proc_easy::easy_argument! {
    #[derive(Clone, Copy)]
    pub struct AccelerationStructure {
        pub kw: kw::acceleration_structure,
    }
}

impl AccelerationStructure {
    #[inline]
    pub fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}
