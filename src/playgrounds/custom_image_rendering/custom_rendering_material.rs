use bevy::{
    asset::{Asset, RenderAssetUsages},
    prelude::*,
    render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

const SHADER_PATH: &str = "shaders/custom_image_rendering.wgsl";

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct CustomRenderMaterial {
    #[texture(0, sample_type = "u_int")]
    pub data: Handle<Image>,
}

impl CustomRenderMaterial {
    pub fn new(images: &mut Assets<Image>, height: usize, width: usize) -> Self {
        Self {
            data: images.add(Self::create_texture_data(height, width)),
        }
    }

    pub fn create_texture_data(height: usize, width: usize) -> Image {
        let data = vec![0; height * width];

        let size = Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        };

        Image::new_fill(
            size,
            TextureDimension::D2,
            &data,
            TextureFormat::R8Uint,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
    }
}

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
