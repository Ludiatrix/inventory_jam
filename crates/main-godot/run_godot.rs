#[cfg(not(feature = "itest"))] // Keep this conditional compilation statement if you want to set up integration tests
fn main() {
    gdenv_lib::api::godot_runner::GodotRunner::init()
        .and_then(|r| r.build())
        .and_then(|r| r.execute())
        .unwrap_or_else(gdenv_lib::api::errors::print_error_stack);
}

// Keep the following code block if you want to set up integration tests
// Run with `cargo run --features itest` to run integration tests
#[cfg(feature = "itest")]
fn main() {
    gdenv_lib::api::godot_runner::GodotRunner::init()
        .and_then(|r| {
            r.godot_cli_arguments(Some(vec![
                "--headless".to_string(),
                "--scene".to_string(),
                "res://addons/godot-bevy/test/TestRunner.tscn".to_string(),
                "--quit-after".to_string(),
                "10000".to_string(),
            ]))
                .build()
        })
        .and_then(|r| r.execute())
        .unwrap_or_else(gdenv_lib::api::errors::print_error_stack);

    std::process::exit(godot_bevy_test::exit_code::read_and_cleanup_exit_code().unwrap_or(1));
}
