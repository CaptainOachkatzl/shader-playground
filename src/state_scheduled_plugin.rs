use bevy::{
    ecs::{schedule::ScheduleLabel, system::ScheduleSystem},
    prelude::*,
};

pub trait StateScheduled<S: States> {
    fn active_state() -> S;

    // fn add_system_running_on_enter<M>(
    //     app: &mut App,
    //     system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    // ) {
    //     app.add_systems(OnEnter(Self::active_state()), system);
    // }

    // fn add_system_running_on_exit<M>(
    //     app: &mut App,
    //     system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    // ) {
    //     app.add_systems(OnExit(Self::active_state()), system);
    // }

    // /// add a system that will only run if the application is in the [active state](Self::active_state)
    // fn add_systems_running_in_state<M>(
    //     app: &mut App,
    //     schedule: impl ScheduleLabel,
    //     system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    // ) {
    //     app.add_systems(schedule, system.run_if(in_state(Self::active_state())));
    // }
}

pub trait AppStateScheduledExt {
    fn add_system_running_on_exit<M>(
        &mut self,
        state: impl States,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self;

    fn add_system_running_on_enter<M>(
        &mut self,
        state: impl States,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self;
}

impl AppStateScheduledExt for App {
    fn add_system_running_on_exit<M>(
        &mut self,
        state: impl States,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.add_systems(OnExit(state), system);
        self
    }

    fn add_system_running_on_enter<M>(
        &mut self,
        state: impl States,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.add_systems(OnEnter(state), system);
        self
    }
}

pub trait StatesExt {
    fn add_system_running_on_enter<M>(
        &self,
        app: &mut App,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    );
}

impl<T: States> StatesExt for T {
    fn add_system_running_on_enter<M>(
        &self,
        app: &mut App,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) {
        app.add_systems(OnEnter(self.clone()), system);
    }
}

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum TestState {
    A,
    B,
}

struct TestPlugin;
impl Plugin for TestPlugin {
    fn build(&self, app: &mut App) {

        TestState::A.add_system_running_on_enter(app, test_system);

        app.add_systems(OnEnter(TestState::A), test_system);

        app.add_system_running_on_enter(TestState::A, test_system);
    }
}

impl StateScheduled<TestState> for TestPlugin {
    fn active_state() -> TestState {
        TestState::A
    }
}

fn test_system() {}
fn test_system_with_params(_: Commands) {}
