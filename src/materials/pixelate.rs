use bevy::{
    prelude::*,
    reflect::{TypeUuid, TypePath},
    render::render_resource::{
            AsBindGroup,
            ShaderRef
        },
    sprite::Material2d,
};

#[derive(AsBindGroup, TypeUuid, Clone, TypePath)]
#[uuid = "d5c3714e-023e-4906-94a2-94e2771b30e1"]
pub struct PixelateMaterial {
    #[uniform(0)]
    pub pixelsize: Vec2,
    #[texture(1)]
    #[sampler(2)]
    pub source_image: Handle<Image>,
}

impl Material2d for PixelateMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/pixelate.wgsl".into()
    }
}