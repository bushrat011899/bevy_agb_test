use bevy_app::{App, AppExit, Plugin};

#[derive(Default)]
pub struct AgbRunnerPlugin;

impl Plugin for AgbRunnerPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.set_runner(agb_runner);
    }
}

pub fn agb_runner(mut app: App) -> AppExit {
    let vblank = agb::interrupt::VBlank::get();

    loop {
        app.update();

        if let Some(exit) = app.should_exit() {
            return exit;
        }

        vblank.wait_for_vblank();
    }
}
