#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
#![deny(clippy::all)]

extern crate alloc;

mod input;
mod logging;
mod render;
mod runner;
mod time;

pub mod prelude {
    pub use bevy_app::prelude::*;
    pub use bevy_color::prelude::*;
    pub use bevy_ecs::prelude::*;
    pub use bevy_input::prelude::*;
    pub use bevy_math::prelude::*;
    pub use bevy_platform_support::prelude::*;
    pub use bevy_state::prelude::*;
    pub use bevy_tasks::prelude::*;
    pub use bevy_time::prelude::*;
    pub use bevy_transform::prelude::*;
    pub use bevy_utils::prelude::*;
}

pub use input::*;
pub use logging::*;
pub use render::*;
pub use runner::*;
pub use time::*;

use bevy_app::{plugin_group, prelude::*};
use bevy_ecs::prelude::*;
use bevy_platform_support::prelude::*;

plugin_group! {
    /// This plugin group will add all the default plugins for a *Bevy* application using [`agb`].
    pub struct AgbPlugin {
        :AgbUnpackPlugin,
        :AgbLogPlugin,
        :AgbInputPlugin,
        :AgbRenderPlugin,
        :AgbRunnerPlugin,
        :AgbTimePlugin,
    }
}

/// Manages access to the Game Boy Advance's beeps and boops sound hardware as part of the
/// original Game Boy's sound chip (the DMG).
#[derive(Resource)]
pub struct Sound(pub agb::sound::dmg::Sound);

/// Manages access to the Game Boy Advance's direct sound mixer for playing raw wav files.
#[derive(Resource)]
pub struct MixerController(pub agb::sound::mixer::MixerController);

/// Manages access to the Game Boy Advance cartridge's save chip.
#[derive(Resource)]
pub struct SaveManager(pub agb::save::SaveManager);

#[derive(Default)]
pub struct AgbUnpackPlugin;

impl Plugin for AgbUnpackPlugin {
    fn build(&self, app: &mut App) {
        // SAFETY: Must ensure this plugin is only added and built once.
        let gba = unsafe { agb::Gba::new_in_entry() };

        // Unpack agb structs into Bevy resources
        let agb::Gba {
            display,
            sound,
            mixer,
            save,
            mut timers,
            dma,
            ..
        } = gba;
        let agb::display::Display {
            video,
            object,
            window,
            blend,
            ..
        } = display;
        let agb::timer::Timers { timer2, timer3, .. } = timers.timers();

        let object = Box::leak(Box::new(object));

        let (unmanaged, sprites) = object.get_unmanaged();

        app.insert_non_send_resource(unmanaged);
        app.insert_non_send_resource(sprites);

        app.insert_resource(Video(video))
            .insert_resource(WindowDist(window))
            .insert_resource(BlendDist(blend))
            .insert_resource(Sound(sound))
            .insert_resource(MixerController(mixer))
            .insert_resource(SaveManager(save))
            .insert_resource(Timer::<2>(timer2))
            .insert_resource(Timer::<3>(timer3))
            .insert_resource(DmaController(dma));
    }
}
