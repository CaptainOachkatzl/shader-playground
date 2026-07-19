use bevy::{
    ecs::{schedule::ScheduleLabel, system::ScheduleSystem},
    prelude::*,
};

pub trait StateScheduledPlugin<S: States> {
    fn active_state() -> S;

    fn add_system_running_on_enter<M>(
        app: &mut App,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) {
        app.add_systems(OnEnter(Self::active_state()), system);
    }

    fn add_system_running_on_exit<M>(
        app: &mut App,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) {
        app.add_systems(OnExit(Self::active_state()), system);
    }

    /// add a system that will only run if the application is in the [active state](Self::active_state)
    fn add_systems_running_in_state<M>(
        app: &mut App,
        schedule: impl ScheduleLabel,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) {
        app.add_systems(schedule, system.run_if(in_state(Self::active_state())));
    }
}

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum TestStates {
    State0,
    State1,
}

struct TestPlugin;
impl Plugin for TestPlugin {
    fn build(&self, app: &mut App) {
        Self::add_system_running_on_enter(app, test_system_with_params);
        Self::add_systems_running_in_state(app, Update, test_system);
    }
}

impl StateScheduledPlugin<TestStates> for TestPlugin {
    fn active_state() -> TestStates {
        TestStates::State0
    }
}

fn test_system() {}
fn test_system_with_params(_: Commands) {}
