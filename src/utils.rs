use bevy::{
    ecs::system::{Commands, Res},
    render::Extract,
    state::state::{State, States},
};

pub fn extract_state<S: States>(mut commands: Commands, state: Extract<Res<State<S>>>) {
    commands.insert_resource(State::new(state.get().clone()));
}
