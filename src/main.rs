use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
#[cfg(feature = "gui")]
use bevy::window::PresentMode;
#[cfg(feature = "gui")]
use bevy::winit::WinitSettings;
use inventory_jam::app::*;
#[cfg(all(feature = "client", not(feature = "gui")))]
use inventory_jam::bot::BotPlugin;
use inventory_jam::combat::CombatPlugin;
use inventory_jam::debug_stats::DebugStatsPlugin;
use inventory_jam::enemy::EnemyPlugin;
use inventory_jam::fragment::FragmentPlugin;
use inventory_jam::gate::GatePlugin;
use inventory_jam::networking;
use inventory_jam::persistence::PersistencePlugin;
use inventory_jam::player::PlayerPlugin;
use inventory_jam::projectile::ProjectilePlugin;
#[cfg(feature = "server")]
use inventory_jam::server::ExampleServerPlugin;
use inventory_jam::settings::{GameSettings, GameSettingsLoadError};
use inventory_jam::shared::SharedPlugin;
use inventory_jam::ui::UiPlugin;
use inventory_jam::weapon::WeaponPlugin;
use inventory_jam::weapon_station::WeaponStationPlugin;
use inventory_jam::world::WorldPlugin;
use std::{process::ExitCode, time::Duration};

const TICK_DURATION: Duration =
    Duration::from_nanos((1_000_000_000.0 / networking::FIXED_TIMESTEP_HZ) as u64);

#[cfg(not(feature = "gui"))]
const HEADLESS_CLIENT_LOOP_HZ: f64 = 60.0;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Application startup failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), GameSettingsLoadError> {
    let settings = GameSettings::load("assets/settings")?;

    let config = config_from_env();
    let mut app = base_app();
    app.insert_resource(settings)
        .insert_resource(config.clone())
        .insert_resource(LocalUsername(config.username.clone()));

    #[cfg(feature = "server")]
    app.add_plugins(lightyear::prelude::server::ServerPlugins {
        tick_duration: TICK_DURATION,
    });

    #[cfg(feature = "client")]
    app.add_plugins(lightyear::prelude::client::ClientPlugins {
        tick_duration: TICK_DURATION,
    });

    #[cfg(feature = "client")]
    app.init_state::<ClientState>();

    #[cfg(feature = "server")]
    app.init_state::<ServerState>();

    app.add_plugins((
        WorldPlugin,
        GatePlugin,
        WeaponStationPlugin,
        UiPlugin,
        LaunchPlugin,
        SharedPlugin,
        CombatPlugin,
        PlayerPlugin,
        EnemyPlugin,
        ProjectilePlugin,
        WeaponPlugin,
        FragmentPlugin,
        PersistencePlugin,
        DebugStatsPlugin,
    ));

    #[cfg(all(feature = "client", not(feature = "gui")))]
    app.add_plugins(BotPlugin);

    networking::configure_networking(&mut app);

    #[cfg(feature = "server")]
    app.add_observer(networking::apply_server_link_conditioner)
        .add_plugins(ExampleServerPlugin);

    app.run();
    Ok(())
}

fn base_app() -> App {
    let mut app = App::new();

    #[cfg(feature = "gui")]
    {
        use inventory_jam::shared::GAME_NAME;

        app.add_plugins(
            DefaultPlugins
                .build()
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                })
                .set(bevy::image::ImagePlugin::default_nearest())
                .set(log_plugin())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: GAME_NAME.to_owned(),
                        resolution: (1024, 768).into(),
                        present_mode: PresentMode::AutoVsync,
                        prevent_default_event_handling: true,
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(WinitSettings::continuous());
        #[cfg(feature = "dev")]
        app.add_plugins((
            bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
            bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
        ));
        app
    }

    #[cfg(not(feature = "gui"))]
    {
        app.add_plugins((
            MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(
                Duration::from_secs_f64(1.0 / HEADLESS_CLIENT_LOOP_HZ),
            )),
            TransformPlugin,
            bevy::input::InputPlugin,
            log_plugin(),
            bevy::state::app::StatesPlugin,
            bevy::diagnostic::DiagnosticsPlugin,
        ));
        app
    }
}

fn log_plugin() -> LogPlugin {
    LogPlugin {
        level: Level::INFO,
        filter: std::env::var("INVENTORY_JAM_LOG").unwrap_or_else(|_| {
            "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn".into()
        }),
        ..default()
    }
}
