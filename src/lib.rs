#![allow(clippy::too_many_arguments)]

mod camera;
mod playgrounds;
mod settings;

#[allow(unused)]
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    image::ImageSamplerDescriptor,
    prelude::*,
    window::{PresentMode, WindowResolution},
};

use crate::{
    camera::CameraPlugin,
    playgrounds::{
        ScenePlugin,
        basic_coloring::{BasicColoringMaterial, BasicColoringPlugin},
        compute_shader::ComputeShaderPlugin,
        custom_render_phase::CustomRenderPhasePlugin,
        hologram::{HologramMaterial, HologramPlugin},
        mesh_manipulation::MeshManipulationPlugin,
        paint_cube_face::{PaintCubeFacePlugin, PaintFaceMaterial},
        rainbow_cube::{RainbowCubePlugin, RainbowMaterial},
        specialized_pipeline::SpecializedPipelinePlugin,
    },
};

pub fn run() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoVsync,
                    title: "shader playground".to_string(),
                    resolution: WindowResolution::new(
                        settings::SCREEN_WIDTH as u32,
                        settings::SCREEN_HEIGHT as u32,
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    // .add_plugins((LogDiagnosticsPlugin::default(), FrameTimeDiagnosticsPlugin::default()))
    // thirdparty plugins
    // local plugins
    .add_plugins((CameraPlugin, ScenePlugin))
    .add_plugins((
        BasicColoringPlugin,
        PaintCubeFacePlugin,
        RainbowCubePlugin,
        HologramPlugin,
        CustomRenderPhasePlugin,
        SpecializedPipelinePlugin,
        ComputeShaderPlugin,
        MeshManipulationPlugin,
    ))
    .add_plugins((
        MaterialPlugin::<BasicColoringMaterial>::default(),
        MaterialPlugin::<PaintFaceMaterial>::default(),
        MaterialPlugin::<RainbowMaterial>::default(),
        MaterialPlugin::<HologramMaterial>::default(),
    ))
    .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
    .insert_resource(Time::<Fixed>::from_hz(60.))
    .run();
}

pub fn is_debug_mode() -> bool {
    cfg!(debug_assertions)
}
