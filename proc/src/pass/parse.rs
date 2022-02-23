use std::{collections::HashSet, convert::TryFrom as _};

use syn::{parse::ParseStream, spanned::Spanned as _};

use crate::{find_unique, find_unique_attribute, kw, take_attributes, validate_member};

pub struct Input {
    pub item_struct: syn::ItemStruct,
    pub attachments: Vec<Attachment>,
    pub subpasses: Vec<Subpass>,
}

#[derive(Clone)]
pub enum Layout {
    Expr(Box<syn::Expr>),
    Member(Box<syn::Member>),
}

#[derive(Clone)]
pub enum ClearValue {
    Expr(Box<syn::Expr>),
    Member(Box<syn::Member>),
}

#[derive(Clone)]
pub enum LoadOp {
    DontCare,
    Clear(ClearValue),
    Load(Layout),
}

#[derive(Clone)]
pub enum StoreOp {
    DontCare,
    Store(Layout),
}

pub struct Attachment {
    pub member: syn::Member,
    pub ty: syn::Type,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
}

impl Attachment {
    fn validate(&self, item_struct: &syn::ItemStruct) -> syn::Result<()> {
        match &self.load_op {
            LoadOp::Load(Layout::Member(layout)) => {
                validate_member(layout, item_struct)?;
            }
            LoadOp::Clear(ClearValue::Member(value)) => {
                validate_member(value, item_struct)?;
            }
            _ => {}
        }
        if let StoreOp::Store(Layout::Member(layout)) = &self.store_op {
            validate_member(layout, item_struct)?;
        }
        Ok(())
    }
}

pub struct Subpass {
    pub colors: Vec<u32>,
    pub depth: Option<u32>,
}

struct SubpassAttribute {
    pub colors: Vec<syn::Member>,
    pub depth: Option<syn::Member>,
}

impl SubpassAttribute {
    fn convert(
        &self,
        attachments: &[Attachment],
        item_struct: &syn::ItemStruct,
    ) -> syn::Result<Subpass> {
        let mut unique = HashSet::with_capacity(self.colors.len() + self.depth.is_some() as usize);

        let mut color_indices = Vec::with_capacity(self.colors.len());
        let mut depth_index = None;

        for color in &self.colors {
            if !unique.insert(color) {
                return Err(syn::Error::new_spanned(
                    color,
                    "Duplicate attachment references are not allowed",
                ));
            }

            validate_member(color, item_struct)?;

            match attachments.iter().position(|a| a.member == *color) {
                Some(index) => {
                    let index = u32::try_from(index).unwrap();

                    color_indices.push(index);
                }
                None => {
                    return Err(syn::Error::new_spanned(
                        color,
                        "Member is not an attachment",
                    ))
                }
            }
        }
        for depth in &self.depth {
            if !unique.insert(depth) {
                return Err(syn::Error::new_spanned(
                    depth,
                    "Duplicate attachment references are not allowed",
                ));
            }
            validate_member(depth, item_struct)?;

            match attachments.iter().position(|a| a.member == *depth) {
                Some(index) => {
                    let index = u32::try_from(index).unwrap();
                    depth_index = Some(index);
                }
                None => {
                    return Err(syn::Error::new_spanned(
                        depth,
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

pub fn parse(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> syn::Result<Input> {
    if !attr.is_empty() {
        return Err(syn::Error::new_spanned(
            proc_macro2::TokenStream::from(attr),
            "#[pass] attribute does not accept arguments",
        ));
    }

    let mut item_struct = syn::parse::<syn::ItemStruct>(item)?;

    let mut attachments = Vec::with_capacity(item_struct.fields.len());

    for (i, f) in item_struct.fields.iter_mut().enumerate() {
        let i = match u32::try_from(i) {
            Ok(i) => i,
            Err(_) => {
                return Err(syn::Error::new_spanned(f, "Too many fields"));
            }
        };

        attachments.extend(parse_attachment(f, i)?);
    }

    let subpass_attrs = take_attributes(&mut item_struct.attrs, parse_subpass_attr)?;

    for attachment in &attachments {
        attachment.validate(&item_struct)?;
    }

    let mut subpasses = Vec::with_capacity(subpass_attrs.len());

    for subpass in &subpass_attrs {
        subpasses.push(subpass.convert(&attachments, &item_struct)?);
    }

    if subpasses.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "At least one subpass attribute must be specified",
        ));
    }

    Ok(Input {
        item_struct,
        attachments,
        subpasses,
    })
}

enum SubpassArg {
    Color { member: syn::Member },
    Depth { kw: kw::depth, member: syn::Member },
}

fn parse_subpass_attr(attr: &syn::Attribute) -> syn::Result<Option<SubpassAttribute>> {
    match attr.path.get_ident() {
        Some(ident) if ident == "subpass" => {}
        _ => return Ok(None),
    }

    let args = attr
        .parse_args_with(|stream: syn::parse::ParseStream<'_>| {
            stream.parse_terminated::<_, syn::Token![,]>(|stream: syn::parse::ParseStream<'_>| {
                let lookahead1 = stream.lookahead1();
                if lookahead1.peek(kw::color) {
                    let _kw = stream.parse::<kw::color>()?;
                    let _eq = stream.parse::<syn::Token![=]>()?;
                    let member = stream.parse::<syn::Member>()?;
                    Ok(SubpassArg::Color { member })
                } else if lookahead1.peek(kw::depth) {
                    let kw = stream.parse::<kw::depth>()?;
                    let _eq = stream.parse::<syn::Token![=]>()?;
                    let member = stream.parse::<syn::Member>()?;
                    Ok(SubpassArg::Depth { kw, member })
                } else {
                    Err(lookahead1.error())
                }
            })
        })
        .unwrap();

    let mut colors = Vec::new();
    let mut depth = None;

    for arg in args {
        match arg {
            SubpassArg::Color { member, .. } => colors.push(member),
            SubpassArg::Depth { kw, member, .. } => {
                if depth.is_some() {
                    return Err(syn::Error::new_spanned(
                        kw,
                        "At most one `depth` argument for `subpass` attribute can be specified",
                    ));
                }
                depth = Some(member);
            }
        }
    }

    Ok(Some(SubpassAttribute { colors, depth }))
}

fn parse_attachment(field: &mut syn::Field, field_index: u32) -> syn::Result<Option<Attachment>> {
    let attachment = find_unique_attribute(
        &mut field.attrs,
        parse_attachment_attr,
        "At most one `attachment` attribute can be specified",
    )?;

    match attachment {
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

struct AttachmentAttribute {
    load_op: LoadOp,
    store_op: StoreOp,
}

enum AttachmentAttributeArgument {
    LoadOp(LoadOp),
    StoreOp(StoreOp),
}

fn parse_attachment_attr(attr: &syn::Attribute) -> syn::Result<Option<AttachmentAttribute>> {
    if attr.path.get_ident().map_or(true, |i| i != "attachment") {
        Ok(None)
    } else {
        let mut load_op = LoadOp::DontCare;
        let mut store_op = StoreOp::DontCare;

        if !attr.tokens.is_empty() {
            let args = attr.parse_args_with(|stream: ParseStream| {
                stream.parse_terminated::<_, syn::Token![,]>(|stream: ParseStream| {
                    let lookahead1 = stream.lookahead1();

                    if lookahead1.peek(kw::clear) {
                        let _kw = stream.parse::<kw::clear>()?;
                        let value;
                        syn::parenthesized!(value in stream);

                        let value = if value.peek(syn::Token![const]) {
                            let _const = value.parse::<syn::Token![const]>()?;
                            let expr = value.parse::<syn::Expr>()?;
                            ClearValue::Expr(Box::new(expr))
                        } else {
                            let member = value.parse::<syn::Member>()?;
                            ClearValue::Member(Box::new(member))
                        };

                        Ok(AttachmentAttributeArgument::LoadOp(LoadOp::Clear(value)))
                    } else if lookahead1.peek(kw::load) {
                        let _kw = stream.parse::<kw::load>()?;
                        let value;
                        syn::parenthesized!(value in stream);

                        let layout = if value.peek(syn::Token![const]) {
                            let _const = value.parse::<syn::Token![const]>()?;
                            let expr = value.parse::<syn::Expr>()?;
                            Layout::Expr(Box::new(expr))
                        } else {
                            let member = value.parse::<syn::Member>()?;
                            Layout::Member(Box::new(member))
                        };

                        Ok(AttachmentAttributeArgument::LoadOp(LoadOp::Load(layout)))
                    } else if lookahead1.peek(kw::store) {
                        let _kw = stream.parse::<kw::store>()?;
                        let value;
                        syn::parenthesized!(value in stream);

                        let layout = if value.peek(syn::Token![const]) {
                            let _const = value.parse::<syn::Token![const]>()?;
                            let expr = value.parse::<syn::Expr>()?;
                            Layout::Expr(Box::new(expr))
                        } else {
                            let member = value.parse::<syn::Member>()?;
                            Layout::Member(Box::new(member))
                        };

                        Ok(AttachmentAttributeArgument::StoreOp(StoreOp::Store(layout)))
                    } else {
                        Err(lookahead1.error())
                    }
                })
            })?;

            load_op = find_unique(
                args.iter().filter_map(|arg| match arg {
                    AttachmentAttributeArgument::LoadOp(load_op) => Some(load_op),
                    _ => None,
                }),
                attr,
                "`attribute` argument must have at most one `clear` or `load` argument",
            )?
            .cloned()
            .unwrap_or(LoadOp::DontCare);

            store_op = find_unique(
                args.iter().filter_map(|arg| match arg {
                    AttachmentAttributeArgument::StoreOp(store_op) => Some(store_op),
                    _ => None,
                }),
                attr,
                "`attribute` argument must have at most one `clear` or `load` argument",
            )?
            .cloned()
            .unwrap_or(StoreOp::DontCare);
        }

        Ok(Some(AttachmentAttribute { load_op, store_op }))
    }
}

// fn parse_clear_value(stream: ParseStream) -> syn::Result<ClearValue> {
//     if stream.fork().parse::<syn::LitInt>().is_ok() {
//         let s = stream.parse::<syn::LitInt>()?.base10_parse::<u32>()?;
//         Ok(ClearValue::DepthStencil(0.0, s))
//     } else {
//         let r_or_d = stream.parse::<syn::LitFloat>()?.base10_parse::<f32>()?;
//         if stream.peek(syn::Token![,]) {
//             stream.parse::<syn::Token![,]>()?;
//             if stream.is_empty() {
//                 Ok(ClearValue::DepthStencil(r_or_d, 0))
//             } else if stream.fork().parse::<syn::LitInt>().is_ok() {
//                 let s = stream.parse::<syn::LitInt>()?.base10_parse::<u32>()?;
//                 Ok(ClearValue::DepthStencil(r_or_d, s))
//             } else {
//                 let g = stream.parse::<syn::LitFloat>()?.base10_parse::<f32>()?;
//                 stream.parse::<syn::Token![,]>()?;
//                 let b = stream.parse::<syn::LitFloat>()?.base10_parse::<f32>()?;
//                 stream.parse::<syn::Token![,]>()?;
//                 let a = stream.parse::<syn::LitFloat>()?.base10_parse::<f32>()?;

//                 if stream.peek(syn::Token![,]) {
//                     stream.parse::<syn::Token![,]>()?;
//                 }

//                 Ok(ClearValue::Color(r_or_d, g, b, a))
//             }
//         } else {
//             Ok(ClearValue::DepthStencil(r_or_d, 0))
//         }
//     }
// }
