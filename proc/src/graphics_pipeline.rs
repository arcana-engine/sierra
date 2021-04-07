use proc_macro2::TokenStream;

pub fn graphics_pipeline(tokens: proc_macro::TokenStream) -> TokenStream {
    match parse(tokens) {
        Ok(input) => {
            let default = syn::parse_str("Default::default()").unwrap();

            let rasterizer = match (
                &input.rasterizer,
                &input.viewport,
                &input.scissor,
                &input.depth_clamp,
                &input.front_face,
                &input.culling,
                &input.polygon_mode,
                &input.depth_test,
                &input.stencil_tests,
                &input.depth_bounds,
                &input.fragment_shader,
                &input.color_blend,
            ) {
                (Some(_), Some(field), _, _, _, _, _, _, _, _, _, _)
                | (Some(_), _, Some(field), _, _, _, _, _, _, _, _, _)
                | (Some(_), _, _, Some(field), _, _, _, _, _, _, _, _)
                | (Some(_), _, _, _, Some(field), _, _, _, _, _, _, _)
                | (Some(_), _, _, _, _, Some(field), _, _, _, _, _, _)
                | (Some(_), _, _, _, _, _, Some(field), _, _, _, _, _)
                | (Some(_), _, _, _, _, _, _, Some(field), _, _, _, _)
                | (Some(_), _, _, _, _, _, _, _, Some(field), _, _, _)
                | (Some(_), _, _, _, _, _, _, _, _, Some(field), _, _)
                | (Some(_), _, _, _, _, _, _, _, _, _, Some(field), _)
                | (Some(_), _, _, _, _, _, _, _, _, _, _, Some(field)) => {
                    return syn::Error::new_spanned(
                        field,
                        "`rasterizer` field must not be specified with any of its subfields",
                    )
                    .to_compile_error();
                }
                (None, None, None, None, None, None, None, None, None, None, None, None) => {
                    quote::quote! {
                        ::std::option::Option::None
                    }
                }
                (
                    Some(rasterizer),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ) => {
                    quote::quote! {
                        ::std::option::Option::Some(#rasterizer)
                    }
                }
                (
                    None,
                    viewport,
                    scissor,
                    depth_clamp,
                    front_face,
                    culling,
                    polygon_mode,
                    depth_test,
                    stencil_tests,
                    depth_bounds,
                    fragment_shader,
                    color_blend,
                ) => {
                    let viewport = viewport.as_ref().unwrap_or(&default);
                    let scissor = scissor.as_ref().unwrap_or(&default);
                    let depth_clamp = depth_clamp.as_ref().unwrap_or(&default);
                    let front_face = front_face.as_ref().unwrap_or(&default);
                    let culling = culling.as_ref().unwrap_or(&default);
                    let polygon_mode = polygon_mode.as_ref().unwrap_or(&default);
                    let depth_test = depth_test.as_ref().unwrap_or(&default);
                    let stencil_tests = stencil_tests.as_ref().unwrap_or(&default);
                    let depth_bounds = depth_bounds.as_ref().unwrap_or(&default);
                    let fragment_shader = fragment_shader.as_ref().unwrap_or(&default);
                    let color_blend = color_blend.as_ref().unwrap_or(&default);

                    quote::quote! {
                        ::std::option::Option::Some(::sierra::Rasterizer {
                            viewport: #viewport,
                            scissor: #scissor,
                            depth_clamp: #depth_clamp,
                            front_face: #front_face,
                            culling: #culling,
                            polygon_mode: #polygon_mode,
                            depth_test: #depth_test,
                            stencil_tests: #stencil_tests,
                            depth_bounds: #depth_bounds,
                            fragment_shader: #fragment_shader,
                            color_blend: #color_blend,
                        })
                    }
                }
            };

            let vertex_bindings = input.vertex_bindings.as_ref().unwrap_or(&default);
            let vertex_attributes = input.vertex_attributes.as_ref().unwrap_or(&default);
            let primitive_topology = input.primitive_topology.as_ref().unwrap_or(&default);
            let primitive_restart_enable =
                input.primitive_restart_enable.as_ref().unwrap_or(&default);
            let vertex_shader = &input.vertex_shader;
            let layout = &input.layout;
            let render_pass = &input.render_pass;
            let subpass = input.subpass.as_ref().unwrap_or(&default);

            quote::quote!(::sierra::GraphicsPipelineInfo {
                vertex_bindings: #vertex_bindings,
                vertex_attributes: #vertex_attributes,
                primitive_topology: #primitive_topology,
                primitive_restart_enable: #primitive_restart_enable,
                vertex_shader: #vertex_shader,
                rasterizer: #rasterizer,
                layout: #layout,
                render_pass: #render_pass,
                subpass: #subpass,
            })
        }
        Err(err) => err.into_compile_error(),
    }
}

struct GraphicsPipelineInput {
    vertex_bindings: Option<syn::Expr>,
    vertex_attributes: Option<syn::Expr>,
    primitive_topology: Option<syn::Expr>,
    primitive_restart_enable: Option<syn::Expr>,
    vertex_shader: syn::Expr,
    rasterizer: Option<syn::Expr>,
    layout: syn::Expr,
    render_pass: syn::Expr,
    subpass: Option<syn::Expr>,

    viewport: Option<syn::Expr>,
    scissor: Option<syn::Expr>,
    depth_clamp: Option<syn::Expr>,
    front_face: Option<syn::Expr>,
    culling: Option<syn::Expr>,
    polygon_mode: Option<syn::Expr>,
    depth_test: Option<syn::Expr>,
    stencil_tests: Option<syn::Expr>,
    depth_bounds: Option<syn::Expr>,
    fragment_shader: Option<syn::Expr>,
    color_blend: Option<syn::Expr>,
}

fn parse(tokens: proc_macro::TokenStream) -> syn::Result<GraphicsPipelineInput> {
    let fields = syn::parse::Parser::parse(
        |stream: syn::parse::ParseStream| {
            stream.parse_terminated::<_, syn::Token![,]>(|stream| stream.parse::<syn::FieldValue>())
        },
        tokens,
    )?;

    let mut vertex_bindings = None;
    let mut vertex_attributes = None;
    let mut primitive_topology = None;
    let mut primitive_restart_enable = None;
    let mut vertex_shader = None;
    let mut rasterizer = None;
    let mut layout = None;
    let mut render_pass = None;
    let mut subpass = None;

    let mut viewport = None;
    let mut scissor = None;
    let mut depth_clamp = None;
    let mut front_face = None;
    let mut culling = None;
    let mut polygon_mode = None;
    let mut depth_test = None;
    let mut stencil_tests = None;
    let mut depth_bounds = None;
    let mut fragment_shader = None;
    let mut color_blend = None;

    for field in fields {
        match &*field.attrs {
            [] => {}
            [attr, ..] => return Err(syn::Error::new_spanned(attr, "Attributes are not expected")),
        }

        match &field.member {
            syn::Member::Named(member) if member == "vertex_bindings" => { vertex_bindings = Some(field.expr); }
            syn::Member::Named(member) if member == "vertex_attributes" => { vertex_attributes = Some(field.expr); }
            syn::Member::Named(member) if member == "primitive_topology" => { primitive_topology = Some(field.expr); }
            syn::Member::Named(member) if member == "primitive_restart_enable" => { primitive_restart_enable = Some(field.expr); }
            syn::Member::Named(member) if member == "vertex_shader" => { vertex_shader = Some(field.expr); }
            syn::Member::Named(member) if member == "rasterizer" => { rasterizer = Some(field.expr); }
            syn::Member::Named(member) if member == "layout" => { layout = Some(field.expr); }
            syn::Member::Named(member) if member == "render_pass" => { render_pass = Some(field.expr); }
            syn::Member::Named(member) if member == "subpass" => { subpass = Some(field.expr); }
            syn::Member::Named(member) if member == "viewport" => { viewport = Some(field.expr); }
            syn::Member::Named(member) if member == "scissor" => { scissor = Some(field.expr); }
            syn::Member::Named(member) if member == "depth_clamp" => { depth_clamp = Some(field.expr); }
            syn::Member::Named(member) if member == "front_face" => { front_face = Some(field.expr); }
            syn::Member::Named(member) if member == "culling" => { culling = Some(field.expr); }
            syn::Member::Named(member) if member == "polygon_mode" => { polygon_mode = Some(field.expr); }
            syn::Member::Named(member) if member == "depth_test" => { depth_test = Some(field.expr); }
            syn::Member::Named(member) if member == "stencil_tests" => { stencil_tests = Some(field.expr); }
            syn::Member::Named(member) if member == "depth_bounds" => { depth_bounds = Some(field.expr); }
            syn::Member::Named(member) if member == "fragment_shader" => { fragment_shader = Some(field.expr); }
            syn::Member::Named(member) if member == "color_blend" => { color_blend = Some(field.expr); }

            member => {
                return Err(syn::Error::new_spanned(
                    member,
                    "Expected only fields named \"vertex_bindings\", \"vertex_attributes\", \"primitive_topology\", \"primitive_restart_enable\", \"vertex_shader\", \"rasterizer\", \"layout\", \"render_pass\" and \"subpass\"",
                ))
            }
        }
    }

    Ok(GraphicsPipelineInput {
        vertex_bindings,
        vertex_attributes,
        primitive_topology,
        primitive_restart_enable,
        vertex_shader: vertex_shader.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing `vertex_shader` field",
            )
        })?,
        rasterizer,

        layout: layout.ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "Missing `layout` field")
        })?,
        render_pass: render_pass.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing `render_pass` field",
            )
        })?,
        subpass,
        viewport,
        scissor,
        depth_clamp,
        front_face,
        culling,
        polygon_mode,
        depth_test,
        stencil_tests,
        depth_bounds,
        fragment_shader,
        color_blend,
    })
}
