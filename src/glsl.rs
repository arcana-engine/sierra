use {
    crate::{repr::*, shader::ShaderStage},
    std::{any::TypeId, borrow::Cow, collections::HashSet},
};

/// Provides shader input
pub trait ShaderInputDecl {
    fn glsl(stage: ShaderStage) -> String;

    fn include_glsl(stage: ShaderStage, code: &str) -> String {
        code.lines()
            .map(|line| {
                let include_here = line
                    .strip_prefix("#include \"sierra_generated.h\"")
                    .is_some();

                if include_here {
                    Cow::Owned(Self::glsl(stage))
                } else {
                    Cow::Borrowed(line)
                }
            })
            .collect()
    }
}

pub struct GlslTypeContext {
    generated: HashSet<TypeId>,
    code: String,
}

impl GlslTypeContext {
    pub fn new() -> Self {
        GlslTypeContext {
            generated: HashSet::new(),
            code: String::new(),
        }
    }

    pub fn add<T, U>(&mut self)
    where
        T: GlslType,
        U: Into<T>,
    {
        if self.generated.insert(TypeId::of::<T>()) {
            T::deps(self);
            self.code += "\n";
            self.code += &T::def();
        }
    }

    pub fn code(self) -> String {
        self.code
    }
}

/// Generates declarations for struct in glsl shaders.
pub trait GlslType: 'static {
    fn name() -> &'static str;

    fn suffix() -> String {
        String::new()
    }

    fn deps(_ctx: &mut GlslTypeContext) {}

    fn def() -> String {
        String::new()
    }
}

macro_rules! builtin_glsl_type {
    ($ty:ty as $name:ident $(($($suffix:tt)+))?) => {
        impl GlslType for $ty {
            fn name() -> &'static str {
                std::stringify!($name)
            }

            $(
                fn suffix() -> String {
                    std::stringify!($($suffix)+).to_owned()
                }
            )?
        }
    };
    ($ty:ty as $name:expr) => {
        impl GlslType for $ty {
            fn name() -> &'static str {
                $name
            }
        }
    };
    ($($ty:ty as $name:tt $(($($suffix:tt)+))?),* $(,)?) => {
        $(builtin_glsl_type!($ty as $name $(($($suffix)+))?);)*
    };
}

builtin_glsl_type!(
    f32 as float,
    f64 as double,
    i32 as int,
    u32 as uint,
    i16 as short,
    u16 as ushort,
    i8 as char,
    bool as bool,
);

impl<T> GlslType for [T]
where
    T: GlslType,
{
    fn name() -> &'static str {
        T::name()
    }

    fn suffix() -> String {
        "[]".to_owned()
    }

    fn deps(ctx: &mut GlslTypeContext) {
        T::deps(ctx)
    }

    fn def() -> String {
        String::new()
    }
}

impl<T, const N: usize> GlslType for [T; N]
where
    T: GlslType,
{
    fn name() -> &'static str {
        T::name()
    }

    fn suffix() -> String {
        format!("[{}]", N)
    }

    fn deps(ctx: &mut GlslTypeContext) {
        T::deps(ctx)
    }

    fn def() -> String {
        String::new()
    }
}

builtin_glsl_type!(vec2<f32> as vec2);
builtin_glsl_type!(vec3<f32> as vec3);
builtin_glsl_type!(vec4<f32> as vec4);
builtin_glsl_type!(mat2x2<f32> as mat2x2);
builtin_glsl_type!(mat3x2<f32> as mat3x2);
builtin_glsl_type!(mat4x2<f32> as mat4x2);
builtin_glsl_type!(mat2x3<f32> as mat2x3);
builtin_glsl_type!(mat3x3<f32> as mat3x3);
builtin_glsl_type!(mat4x3<f32> as mat4x3);
builtin_glsl_type!(mat2x4<f32> as mat2x4);
builtin_glsl_type!(mat3x4<f32> as mat3x4);
builtin_glsl_type!(mat4x4<f32> as mat4x4);

builtin_glsl_type!(vec2<f64> as dvec2);
builtin_glsl_type!(vec3<f64> as dvec3);
builtin_glsl_type!(vec4<f64> as dvec4);
builtin_glsl_type!(mat2x2<f64> as dmat2x2);
builtin_glsl_type!(mat3x2<f64> as dmat3x2);
builtin_glsl_type!(mat4x2<f64> as dmat4x2);
builtin_glsl_type!(mat2x3<f64> as dmat2x3);
builtin_glsl_type!(mat3x3<f64> as dmat3x3);
builtin_glsl_type!(mat4x3<f64> as dmat4x3);
builtin_glsl_type!(mat2x4<f64> as dmat2x4);
builtin_glsl_type!(mat3x4<f64> as dmat3x4);
builtin_glsl_type!(mat4x4<f64> as dmat4x4);

builtin_glsl_type!(vec2<i32> as ivec2);
builtin_glsl_type!(vec3<i32> as ivec3);
builtin_glsl_type!(vec4<i32> as ivec4);
builtin_glsl_type!(mat2x2<i32> as imat2x2);
builtin_glsl_type!(mat3x2<i32> as imat3x2);
builtin_glsl_type!(mat4x2<i32> as imat4x2);
builtin_glsl_type!(mat2x3<i32> as imat2x3);
builtin_glsl_type!(mat3x3<i32> as imat3x3);
builtin_glsl_type!(mat4x3<i32> as imat4x3);
builtin_glsl_type!(mat2x4<i32> as imat2x4);
builtin_glsl_type!(mat3x4<i32> as imat3x4);
builtin_glsl_type!(mat4x4<i32> as imat4x4);

builtin_glsl_type!(vec2<u32> as uvec2);
builtin_glsl_type!(vec3<u32> as uvec3);
builtin_glsl_type!(vec4<u32> as uvec4);
builtin_glsl_type!(mat2x2<u32> as umat2x2);
builtin_glsl_type!(mat3x2<u32> as umat3x2);
builtin_glsl_type!(mat4x2<u32> as umat4x2);
builtin_glsl_type!(mat2x3<u32> as umat2x3);
builtin_glsl_type!(mat3x3<u32> as umat3x3);
builtin_glsl_type!(mat4x3<u32> as umat4x3);
builtin_glsl_type!(mat2x4<u32> as umat2x4);
builtin_glsl_type!(mat3x4<u32> as umat3x4);
builtin_glsl_type!(mat4x4<u32> as umat4x4);

builtin_glsl_type!(vec2<bool> as bvec2);
builtin_glsl_type!(vec3<bool> as bvec3);
builtin_glsl_type!(vec4<bool> as bvec4);
builtin_glsl_type!(mat2x2<bool> as bmat2x2);
builtin_glsl_type!(mat3x2<bool> as bmat3x2);
builtin_glsl_type!(mat4x2<bool> as bmat4x2);
builtin_glsl_type!(mat2x3<bool> as bmat2x3);
builtin_glsl_type!(mat3x3<bool> as bmat3x3);
builtin_glsl_type!(mat4x3<bool> as bmat4x3);
builtin_glsl_type!(mat2x4<bool> as bmat2x4);
builtin_glsl_type!(mat3x4<bool> as bmat3x4);
builtin_glsl_type!(mat4x4<bool> as bmat4x4);
