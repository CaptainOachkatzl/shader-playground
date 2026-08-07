use bevy::{
    asset::Asset,
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

const SHADER_PATH: &str = "shaders/custom_image_rendering.wgsl";

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct CustomRenderMaterial {}

impl Material2d for CustomRenderMaterial {
    // fn vertex_shader() -> ShaderRef {
    //     SHADER_PATH.into()
    // }

    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
