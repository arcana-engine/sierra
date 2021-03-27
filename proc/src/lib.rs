extern crate proc_macro;

mod descriptors;
mod repr;
mod stage;

#[proc_macro_attribute]
pub fn descriptors(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    descriptors::descriptors(attr, item).into()
}

#[proc_macro_attribute]
pub fn shader_repr(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    repr::shader_repr(attr, item).into()
}

fn find_unique<I>(iter: I, msg: &'static str) -> Option<I::Item>
where
    I: IntoIterator,
{
    let mut iter = iter.into_iter();
    let item = iter.next()?;
    if iter.next().is_some() {
        panic!("{}", msg);
    }
    Some(item)
}

fn get_unique<I>(iter: I, msg: &'static str) -> I::Item
where
    I: IntoIterator,
{
    let mut iter = iter.into_iter();
    let item = iter.next().expect(msg);
    if iter.next().is_some() {
        panic!("{}", msg);
    }
    item
}
