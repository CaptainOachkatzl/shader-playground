use bevy::{
    ecs::{
        schedule::{ScheduleConfigs, ScheduleLabel},
        system::ScheduleSystem,
    },
    prelude::*,
};

pub trait StateScheduledPlugin<S: States>: Plugin {
    fn active_state() -> S;

    fn on_enter(&self) -> Vec<ScheduleConfigs<ScheduleSystem>> {
        Vec::new()
    }
    fn on_exit(&self) -> Vec<ScheduleConfigs<ScheduleSystem>> {
        Vec::new()
    }

    /// needs to be called during [`Plugin::build`]
    fn init(&self, app: &mut App) {
        for on_enter in self.on_enter() {
            app.add_systems(OnEnter(Self::active_state()), on_enter);
        }

        for on_exit in self.on_exit() {
            app.add_systems(OnExit(Self::active_state()), on_exit);
        }
    }

    /// add a system that will only run if the application is in the [active state](Self::active_state)
    fn add_systems_running_in_state<M>(
        &self,
        app: &mut App,
        schedule: impl ScheduleLabel,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) {
        app.add_systems(schedule, system.run_if(in_state(Self::active_state())));
    }
}

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum TestStates {
    state0,
    state1,
}

struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&self, app: &mut App) {
        self.init(app);
        self.add_systems_running_in_state(app, Update, test_system);
    }
}

impl StateScheduledPlugin<TestStates> for TestPlugin {
    fn active_state() -> TestStates {
        TestStates::state0
    }

    fn on_enter(&self) -> Vec<ScheduleConfigs<ScheduleSystem>> {
        let mut systems = Vec::new();
        systems.push(test_system.into_configs());
        systems.push(test_system_with_params.into_configs());
        systems
    }
}

fn test_system() {}
fn test_system_with_params(commands: Commands) {}
