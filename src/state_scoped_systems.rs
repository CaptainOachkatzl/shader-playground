/// Adds state-scoped systems to a Bevy [`App`].
///
/// This macro is a convenience wrapper around [`App::add_systems`] for registering
/// systems that are associated with a specific Bevy state. It supports:
///
/// - `OnEnter(system)` - runs the system when entering the specified state.
/// - `OnExit(system)` - runs the system when leaving the specified state.
/// - `RunInState(schedule, system)` - runs the system during the given schedule
///   only while the application is in the specified state.
///
/// The state expression is provided once and automatically applied to all
/// registered systems.
///
/// # Example
///
/// ```rust
/// add_state_scoped_systems!(
///     app,
///     GameState::Playing,
///     OnEnter(setup_level),
///     OnExit(cleanup_level),
///     RunInState(Update, update_game),
/// );
/// ```
///
/// The above expands to:
///
/// ```rust
/// app.add_systems(OnEnter(GameState::Playing), setup_level)
///     .add_systems(OnExit(GameState::Playing), cleanup_level)
///     .add_systems(
///         Update,
///         update_game.run_if(in_state(GameState::Playing)),
///     );
/// ```
///
/// This helps keep state-related system registration concise and ensures that
/// all systems associated with a state use the same state expression.
#[macro_export]
macro_rules! add_state_scoped_systems {
    (
        $app:expr,
        $state:expr,
        $( $kind:ident ( $($args:tt)* ) ),* $(,)?
    ) => {{
        $(
            add_state_scoped_systems!(@item $app, $state, $kind($($args)*));
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
