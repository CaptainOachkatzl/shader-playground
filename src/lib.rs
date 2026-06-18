pub mod camera;
mod paint_cube_face;
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
    paint_cube_face::{PaintCubeFacePlugin, PaintFaceMaterial},
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
            .set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor::nearest(),
            }),
    )
    // .add_plugins((LogDiagnosticsPlugin::default(), FrameTimeDiagnosticsPlugin::default()))
    // thirdparty plugins
    // local plugins
    .add_plugins(CameraPlugin)
    .add_plugins(PaintCubeFacePlugin)
    .add_plugins((MaterialPlugin::<PaintFaceMaterial>::default(),))
    .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
    .insert_resource(Time::<Fixed>::from_hz(60.))
    .run();
}

pub fn is_debug_mode() -> bool {
    cfg!(debug_assertions)
}
