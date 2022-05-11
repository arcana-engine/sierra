use std::{collections::HashSet, convert::TryFrom};

use proc_easy::{EasyAttributes, ReferenceExpr};
use syn::spanned::Spanned;

use crate::{kw, validate_member};

pub struct Input {
    pub item_struct: syn::ItemStruct,
    pub attachments: Vec<Attachment>,
    pub subpasses: Vec<Subpass>,
}

pub struct Attachment {
    pub member: syn::Member,
    pub ty: syn::Type,
    pub load_op: Option<LoadOp>,
    pub store_op: Option<StoreOp>,
}

impl Attachment {
    fn validate(&self, item_struct: &syn::ItemStruct) -> syn::Result<()> {
        match &self.load_op {
            Some(LoadOp::Load(Load(_, ReferenceExpr::Member { member }))) => {
                validate_member(member, item_struct)?;
            }
            Some(LoadOp::Clear(Clear(_, ReferenceExpr::Member { member }))) => {
                validate_member(member, item_struct)?;
            }
            _ => {}
        }
        if let Some(StoreOp::Store(Store(_, ReferenceExpr::Member { member }))) = &self.store_op {
            validate_member(member, item_struct)?;
        }
        Ok(())
    }
}

proc_easy::easy_argument_value! {
    pub struct Clear(pub kw::clear, pub ReferenceExpr);
}

proc_easy::easy_argument_value! {
    pub struct Load(pub kw::load, pub ReferenceExpr);
}

proc_easy::easy_argument_group! {
    pub enum LoadOp {
        Clear(Clear),
        Load(Load),
    }
}

proc_easy::easy_argument_value! {
    pub struct Store(pub kw::store, pub ReferenceExpr);
}

proc_easy::easy_argument_group! {
    pub enum StoreOp {
        Store(Store),
    }
}

proc_easy::easy_argument_tuple! {
    struct AttachmentAttribute {
        attachment: kw::attachment,
        load_op: Option<LoadOp>,
        store_op: Option<StoreOp>,
    }
}

proc_easy::easy_attributes! {
    @(sierra)
    struct FieldAttributes {
        attachment: Option<AttachmentAttribute>,
    }
}

fn parse_attachment(field: &syn::Field, field_index: u32) -> syn::Result<Option<Attachment>> {
    let attrs = FieldAttributes::parse(&field.attrs, field.span())?;

    match attrs.attachment {
        None => Ok(None),
        Some(attachment) => {
            let member = match field.ident.as_ref() {
                None => syn::Member::Unnamed(syn::Index {
                    index: field_index,
                    span: field.span(),
                }),
                Some(field_ident) => syn::Member::Named(field_ident.clone()),
            };

            Ok(Some(Attachment {
                ty: field.ty.clone(),
                member,
                load_op: attachment.load_op,
                store_op: attachment.store_op,
            }))
        }
    }
}

pub struct Subpass {
    pub colors: Vec<u32>,
    pub depth: Option<u32>,
}

proc_easy::easy_argument_value! {
    pub struct Color(pub kw::color, pub syn::Member);
}

proc_easy::easy_argument_value! {
    pub struct Depth(pub kw::depth, pub syn::Member);
}

proc_easy::easy_argument_tuple! {
    struct SubpassArgument {
        subpass: kw::subpass,
        colors: Vec<Color>,
        depth: Option<Depth>,
    }
}

impl SubpassArgument {
    fn convert(
        &self,
        attachments: &[Attachment],
        item_struct: &syn::ItemStruct,
    ) -> syn::Result<Subpass> {
        let mut unique = HashSet::with_capacity(self.colors.len() + self.depth.is_some() as usize);

        let mut color_indices = Vec::with_capacity(self.colors.len());
        let mut depth_index = None;

        for Color(_, member) in self.colors.iter() {
            if !unique.insert(member) {
                return Err(syn::Error::new_spanned(
                    member,
                    "Duplicate attachment references are not allowed",
                ));
            }

            validate_member(member, item_struct)?;

            match attachments.iter().position(|a| a.member == *member) {
                Some(index) => {
                    let index = u32::try_from(index).unwrap();

                    color_indices.push(index);
                }
                None => {
                    return Err(syn::Error::new_spanned(
                        member,
                        "Member is not an attachment",
                    ))
                }
            }
        }

        if let Some(Depth(_, member)) = &self.depth {
            if !unique.insert(member) {
                return Err(syn::Error::new_spanned(
                    member,
                    "Duplicate attachment references are not allowed",
                ));
            }
            validate_member(member, item_struct)?;

            match attachments.iter().position(|a| a.member == *member) {
                Some(index) => {
                    let index = u32::try_from(index).unwrap();
                    depth_index = Some(index);
                }
                None => {
                    return Err(syn::Error::new_spanned(
                        member,
                        "Member is not an attachment",
                    ))
                }
            }
        }

        Ok(Subpass {
            colors: color_indices,
            depth: depth_index,
        })
    }
}

proc_easy::easy_attributes! {
    @(sierra)
    struct RenderPassAttributes {
        subpasses: Vec<SubpassArgument>,
    }
}

pub fn parse(item: proc_macro::TokenStream) -> syn::Result<Input> {
    let item_struct = syn::parse::<syn::ItemStruct>(item)?;

    let mut attachments = Vec::with_capacity(item_struct.fields.len());

    for (i, f) in item_struct.fields.iter().enumerate() {
        let i = match u32::try_from(i) {
            Ok(i) => i,
            Err(_) => {
                return Err(syn::Error::new_spanned(f, "Too many fields"));
            }
        };

        attachments.extend(parse_attachment(f, i)?);
    }

    for attachment in &attachments {
        attachment.validate(&item_struct)?;
    }

    let attrs = RenderPassAttributes::parse(&item_struct.attrs, item_struct.ident.span())?;

    let mut subpasses = Vec::with_capacity(attrs.subpasses.len());

    for subpass in &attrs.subpasses {
        subpasses.push(subpass.convert(&attachments, &item_struct)?);
    }

    if subpasses.is_empty() {
        return Err(syn::Error::new_spanned(
            item_struct.ident,
            "At least one subpass must be specified",
        ));
    }

    Ok(Input {
        item_struct,
        attachments,
        subpasses,
    })
}
