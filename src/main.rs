#![no_std]
#![no_main]

extern crate alloc;

use agb::display::{object::SpriteLoader, palette16::Palette16};
use log::info;

use bevy_diagnostic::{DiagnosticsPlugin, FrameCount, FrameCountPlugin};
use bevy_input::{
    gamepad::{GamepadButtonChangedEvent, GamepadButtonStateChangedEvent, GamepadConnectionEvent},
    InputPlugin,
};
use bevy_state::app::StatesPlugin;
use bevy_time::TimePlugin;

use bevy_agb_test::prelude::*;
use bevy_agb_test::*;

#[export_name = "main"]
pub extern "C" fn main() -> ! {
    App::new()
        .add_plugins(AgbPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(TimePlugin)
        .add_plugins(TransformPlugin)
        .add_plugins(FrameCountPlugin)
        .add_plugins(DiagnosticsPlugin)
        .add_plugins(StatesPlugin)
        .init_resource::<FrameCount>()
        .init_non_send_resource::<Option<Sprites>>()
        .add_systems(Startup, (setup_video, load_sprites, spawn_saw).chain())
        .add_systems(
            Update,
            (log_counter, log_frame_time, log_gamepad_events).chain(),
        )
        .add_systems(FixedUpdate, move_player)
        .run();

    loop {}
}

fn setup_video(mut video: ResMut<Video>) {
    let (_background, mut vram) = video.0.tiled0();

    vram.set_background_palettes(&[Palette16::new([u16::MAX; 16])]);
}

fn log_frame_time(time: Res<Time>) {
    info!("Frame Time: {}us", time.delta().as_micros());
}

fn log_counter(count: Res<FrameCount>) {
    info!("Frame Count: {}", count.0);
}

fn log_gamepad_events(
    mut connection_events: EventReader<GamepadConnectionEvent>,
    mut button_changed_events: EventReader<GamepadButtonChangedEvent>,
    mut button_input_events: EventReader<GamepadButtonStateChangedEvent>,
) {
    for connection_event in connection_events.read() {
        info!("{:?}", connection_event);
    }
    for button_changed_event in button_changed_events.read() {
        info!(
            "{:?} of {} is changed to {}",
            button_changed_event.button, button_changed_event.entity, button_changed_event.value
        );
    }
    for button_input_event in button_input_events.read() {
        info!("{:?}", button_input_event);
    }
}

fn load_sprites(mut loader: NonSendMut<SpriteLoader>, mut sprites: NonSendMut<Option<Sprites>>) {
    static SPRITES: &agb::display::object::Graphics = agb::include_aseprite!("assets/bad.aseprite");

    static SAW: &agb::display::object::Sprite = SPRITES.tags().get("Bad").sprite(0);

    let player = Sprite(loader.get_vram_sprite(SAW));

    *sprites = Some(Sprites { player });
}

struct Sprites {
    player: Sprite,
}

#[derive(Component)]
struct Player;

fn spawn_saw(mut commands: Commands, sprites: NonSend<Option<Sprites>>) {
    let sprites = sprites.as_ref().unwrap();
    commands.spawn((Transform::default(), sprites.player.clone(), Player));
}

fn move_player(mut player: Single<&mut Transform, With<Player>>, gamepad: Single<&Gamepad>) {
    if gamepad.pressed(GamepadButton::DPadUp) {
        player.translation.y -= 1.;
    }

    if gamepad.pressed(GamepadButton::DPadDown) {
        player.translation.y += 1.;
    }

    if gamepad.pressed(GamepadButton::DPadLeft) {
        player.translation.x -= 1.;
    }

    if gamepad.pressed(GamepadButton::DPadRight) {
        player.translation.x += 1.;
    }
}
