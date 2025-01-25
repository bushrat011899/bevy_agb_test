#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
#![deny(clippy::all)]

pub mod logging;

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

use core::{sync::atomic::Ordering, time::Duration};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_input::gamepad::{
    GamepadButton, GamepadConnection, GamepadConnectionEvent, RawGamepadButtonChangedEvent,
    RawGamepadEvent,
};
use bevy_math::Vec3;
use bevy_platform_support::{prelude::*, sync::atomic::AtomicU32, time::Instant};
use bevy_transform::components::GlobalTransform;

pub struct AgbPlugin;

impl Plugin for AgbPlugin {
    fn build(&self, app: &mut App) {
        static TIMER_2_INTERRUPTS: AtomicU32 = AtomicU32::new(0);

        let _ = logging::init();

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
        let agb::timer::Timers {
            mut timer2, timer3, ..
        } = timers.timers();

        // Setup time keeping
        // SAFETY: No allocation performed.
        let interrupt_handler = unsafe {
            agb::interrupt::add_interrupt_handler(agb::interrupt::Interrupt::Timer2, |_| {
                TIMER_2_INTERRUPTS.add(1, Ordering::Release);
            })
        };

        app.insert_non_send_resource(interrupt_handler);

        // SAFETY: She'll be right
        unsafe {
            Instant::set_elapsed(Box::leak(Box::new(
                (|| {
                    let interrupts = TIMER_2_INTERRUPTS.load(Ordering::Acquire);

                    Duration::from_nanos(1_000_000_000 >> 8) * interrupts
                }) as fn() -> Duration,
            )) as *mut _);
        }

        timer2
            .set_enabled(true)
            .set_divider(agb::timer::Divider::Divider1)
            .set_overflow_amount(0xFFFF)
            .set_interrupt(true);

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

        // Get the button manager
        app.insert_resource(ButtonController::new());
        app.world_mut().spawn(GameBoyGamepad);
        app.add_systems(Startup, ButtonController::startup);

        // Simple runner that waits for V-Blank
        app.set_runner(|mut app| {
            let vblank = agb::interrupt::VBlank::get();

            loop {
                app.update();

                if let Some(exit) = app.should_exit() {
                    return exit;
                }

                vblank.wait_for_vblank();
            }
        });

        app.add_systems(First, ButtonController::update)
            .add_systems(Last, render_objects);
    }
}

#[derive(Resource)]
pub struct Video(pub agb::display::video::Video);

#[derive(Resource)]
pub struct WindowDist(pub agb::display::WindowDist);

#[derive(Resource)]
pub struct BlendDist(pub agb::display::BlendDist);

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

/// Provides a single timer.
#[derive(Resource)]
pub struct Timer<const ID: u8>(pub agb::timer::Timer);

/// Manages access to the Game Boy Advance's DMA
#[derive(Resource)]
pub struct DmaController(pub agb::dma::DmaController);

/// Helper to make it easy to get the current state of the GBA's buttons.
#[derive(Resource)]
pub struct ButtonController(pub agb::input::ButtonController);

impl ButtonController {
    #[must_use]
    pub fn new() -> Self {
        Self(agb::input::ButtonController::new())
    }

    pub fn startup(
        gamepad: Single<Entity, With<GameBoyGamepad>>,
        mut events: EventWriter<RawGamepadEvent>,
        mut connection_events: EventWriter<GamepadConnectionEvent>,
    ) {
        let gamepad = gamepad.into_inner();

        let event = GamepadConnectionEvent::new(
            gamepad,
            GamepadConnection::Connected {
                name: "GameBoy Advance Gamepad".to_string(),
                vendor_id: None,
                product_id: None,
            },
        );

        events.send(event.clone().into());
        connection_events.send(event);
    }

    pub fn update(
        mut manager: ResMut<Self>,
        mut events: EventWriter<RawGamepadEvent>,
        mut button_events: EventWriter<RawGamepadButtonChangedEvent>,
        gamepad: Single<Entity, With<GameBoyGamepad>>,
    ) {
        manager.0.update();

        let gamepad = gamepad.into_inner();

        const BUTTONS: [(agb::input::Button, GamepadButton); 10] = [
            (agb::input::Button::A, GamepadButton::East),
            (agb::input::Button::B, GamepadButton::South),
            (agb::input::Button::SELECT, GamepadButton::Select),
            (agb::input::Button::START, GamepadButton::Start),
            (agb::input::Button::RIGHT, GamepadButton::DPadRight),
            (agb::input::Button::LEFT, GamepadButton::DPadLeft),
            (agb::input::Button::UP, GamepadButton::DPadUp),
            (agb::input::Button::DOWN, GamepadButton::DPadDown),
            (agb::input::Button::R, GamepadButton::RightTrigger),
            (agb::input::Button::L, GamepadButton::LeftTrigger),
        ];

        for (agb_button, bevy_button) in BUTTONS {
            let value = if manager.0.is_just_pressed(agb_button) {
                1.0
            } else if manager.0.is_just_released(agb_button) {
                0.0
            } else {
                continue;
            };

            events.send(RawGamepadButtonChangedEvent::new(gamepad, bevy_button, value).into());
            button_events.send(RawGamepadButtonChangedEvent::new(
                gamepad,
                bevy_button,
                value,
            ));
        }
    }
}

#[derive(Component)]
pub struct GameBoyGamepad;

#[derive(Component, Clone)]
pub struct Sprite {
    pub handle: agb::display::object::SpriteVram,
    pub horizontal_flipped: bool,
    pub vertical_flipped: bool,
}

unsafe impl Send for Sprite {}
unsafe impl Sync for Sprite {}

fn render_objects(
    mut oam: NonSendMut<agb::display::object::OamUnmanaged<'static>>,
    sprites: Query<(&Sprite, &GlobalTransform)>,
) {
    let oam_iterator = &mut oam.iter();

    for (sprite, transform) in &sprites {
        let Vec3 { x, y, .. } = transform.translation();

        let x = x as i32;
        let y = y as i32;

        let position = agb::fixnum::Vector2D { x, y };

        let mut obj = agb::display::object::ObjectUnmanaged::new(sprite.handle.clone());
        obj.show()
            .set_position(position)
            .set_hflip(sprite.horizontal_flipped)
            .set_vflip(sprite.vertical_flipped);

        let Some(next) = oam_iterator.next() else {
            return;
        };

        next.set(&obj);
    }
}
