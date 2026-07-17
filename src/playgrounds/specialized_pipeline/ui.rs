use bevy::{
    input_focus::{FocusCause, InputFocus},
    prelude::*,
};

use crate::playgrounds::{PlaygroundScene, specialized_pipeline::ExplosionType};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PlaygroundScene::SpecializedPipeline),
            (setup, bevy::asset::handle_internal_asset_events).chain(),
        )
        .add_systems(
            Update,
            button_system.run_if(in_state(PlaygroundScene::SpecializedPipeline)),
        );
    }
}

fn setup(mut commands: Commands, assets: Res<AssetServer>, state: Res<State<ExplosionType>>) {
    commands.spawn((
        DespawnOnExit(PlaygroundScene::SpecializedPipeline),
        button_setup(&assets, &state),
    ));
}

fn button_setup(asset_server: &AssetServer, explosion_type: &ExplosionType) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Baseline,
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        children![(
            Button,
            Node {
                width: px(130),
                height: px(22),
                border: UiRect::all(px(1)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BackgroundColor(Color::BLACK),
            children![(
                Text::new(get_button_label(explosion_type)),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf").into(),
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
            )]
        )],
    )
}

fn button_system(
    mut input_focus: ResMut<InputFocus>,
    state: Res<State<ExplosionType>>,
    mut next_state: ResMut<NextState<ExplosionType>>,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut Button, &Children),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, interaction, mut button, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            let mut text = text_query.get_mut(children[0]).unwrap();
            input_focus.set(entity, FocusCause::Pressed);

            let next_explosion_type = match **state {
                ExplosionType::ConstantExpansion => ExplosionType::RandomVelocity,
                ExplosionType::RandomVelocity => ExplosionType::ConstantExpansion,
            };
            **text = get_button_label(&next_explosion_type);
            next_state.set(next_explosion_type);

            // The accessibility system's only update the button's state when the `Button` component is marked as changed.
            button.set_changed();
        }
    }
}

fn get_button_label(explosion_type: &ExplosionType) -> String {
    match explosion_type {
        ExplosionType::ConstantExpansion => "Constant expansion".to_string(),
        ExplosionType::RandomVelocity => "Random velocity".to_string(),
    }
}
