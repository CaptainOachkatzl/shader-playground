use bevy::{
    ecs::{schedule::ScheduleLabel, system::ScheduleSystem},
    prelude::*,
};

#[macro_export]
macro_rules! state_scoped {
    (
        $app:expr,
        $state:expr,
        $( $kind:ident ( $($args:tt)* ) ),* $(,)?
    ) => {{
        $(
            state_scoped!(@item $app, $state, $kind($($args)*));
        )*
    }};

    (@item $app:expr, $state:expr, OnEnter($system:expr)) => {
        $app.add_systems(OnEnter($state), $system);
    };

    (@item $app:expr, $state:expr, OnExit($system:expr)) => {
        $app.add_systems(OnExit($state), $system);
    };

    (@item $app:expr, $state:expr, RunInState($schedule:expr, $system:expr)) => {
        $app.add_systems(
            $schedule,
            $system.run_if(in_state($state)),
        );
    };
}

struct TestPlugin;
impl Plugin for TestPlugin {
    fn build(&self, app: &mut App) {
        
        state_scoped!(
            app,
            TestState::A,
            OnEnter(test_system),
            OnExit(test_system),
            RunInState(Update, test_system),
        );

        let mut state_schedule = StateSchedule::new(app, TestState::A);
        state_schedule
            .add_on_enter_systems(test_system)
            .add_on_exit_systems(test_system)
            .add_state_scoped_systems(Update, test_system);

        app.add_systems(OnEnter(TestState::A), test_system)
            .add_systems(OnExit(TestState::A), test_system)
            .add_systems(
                Update,
                test_system_with_params.run_if(in_state(TestState::A)),
            );
        app.add_systems(OnEnter(TestState::A), test_system)
            .add_systems(OnExit(TestState::A), test_system)
            .add_systems(
                Update,
                test_system_with_params.run_if(in_state(TestState::A)),
            );

        TestState::A.add_system_running_on_enter(app, test_system);

        app.add_systems(OnEnter(TestState::A), test_system);

        app.add_system_running_on_enter(TestState::A, test_system);
    }
}

struct StateSchedule<'a, S: States> {
    state: S,
    app: &'a mut App,
}

impl<'a, S: States> StateSchedule<'a, S> {
    fn new(app: &'a mut App, state: S) -> Self {
        Self { state, app }
    }

    fn add_on_enter_systems<M>(
        &mut self,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.app.add_systems(OnEnter(self.state.clone()), system);
        self
    }

    fn add_on_exit_systems<M>(
        &mut self,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.app.add_systems(OnExit(self.state.clone()), system);
        self
    }

    fn add_state_scoped_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.app
            .add_systems(schedule, system.run_if(in_state(self.state.clone())));
        self
    }
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

fn test_system() {}
fn test_system_with_params(_: Commands) {}
