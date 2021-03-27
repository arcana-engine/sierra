#[sierra::descriptors]
pub struct Filter {
    pub sampler: sierra::Sampler,

    #[combined_image_sampler(sampler)]
    #[stages(Fragment)]
    pub albedo: sierra::Image,

    #[combined_image_sampler(sampler)]
    #[stages(Fragment)]
    pub normal_depth: sierra::Image,

    #[combined_image_sampler(sampler)]
    #[stages(Fragment)]
    pub unfiltered: sierra::Image,
}

#[sierra::shader_repr]
pub struct Bar {
    pub transform: sierra::mat3,
    pub pos: sierra::vec3,
    pub fit: f32,
}

#[sierra::shader_repr]
pub struct Foo {
    pub foo: Bar,
}

#[test]
fn test() {
    use {
        core::mem::size_of,
        sierra::{mat3, Repr, Std140, Std430},
    };

    assert!(
        dbg!(size_of::<<mat3 as Repr<Std140>>::Type>())
            >= dbg!(size_of::<<mat3 as Repr<Std430>>::Type>())
    );
}
