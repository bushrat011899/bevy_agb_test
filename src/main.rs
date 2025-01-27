//! An example game written in the Bevy game engine and using the [`agb`] crate to allow running it
//! on the GameBoy Advance.

//! We declare our crate as `no_std`, as the GameBoy Advance doesn't have a port of the standard
//! library.
#![no_std]

//! We also declare the crate as not having a typical `main` function.
//! The `agb-gbafix` tool we use to generate our final ROM file expects an exported
//! function named `main` accepting no arguments and _never_ returning.
//! This is handled by [`main`].
#![no_main]

//! [`agb`] provides a global allocator, allowing us to use items from the [`alloc`] crate.
extern crate alloc;

use agb::display::{object::SpriteLoader, palette16::Palette16};
use log::info;

// Bevy (currently) does not support `no_std` through its main crate `bevy`.
// Instead we ust import each Bevy member-crate individually so we can enable the appropriate
// features, and disable inappropriate ones.
use bevy_diagnostic::{DiagnosticsPlugin, FrameCountPlugin};
use bevy_input::InputPlugin;
use bevy_state::app::StatesPlugin;
use bevy_time::TimePlugin;

// This module is a little convenience to replicate `bevy::prelude`, which we don't
// (currently) have access to.
use bevy_agb_test::prelude::*;
use bevy_agb_test::*;

#[export_name = "main"]
pub extern "C" fn main() -> ! {
    // We can use Bevy's `App` abstraction just like any other Bevy application.
    let mut app = App::new();

    // The first step is to add the `AgbPlugin`.
    // This sets up integration between Bevy and the `agb` abstraction over the GameBoy Advance.
    // This _must_ be done first, as it also sets up `Instant` for us.
    // Otherwise, the `TimePlugin` will fail to initialize.
    app.add_plugins(AgbPlugin);

    // This is effectively Bevy's `DefaultPlugins` slimmed to what is readily available on `no_std`.
    app.add_plugins((
        InputPlugin,
        TimePlugin,
        TransformPlugin,
        FrameCountPlugin,
        DiagnosticsPlugin,
        StatesPlugin,
    ));

    // Unfortunately, we currently don't have a first-party abstraction for assets or rendering.
    // This means getting assets, and rendering them must be done somewhat manually.
    app.init_non_send_resource::<Option<Sprites>>()
        .add_systems(Startup, (setup_video, load_sprites).chain());

    // This is our game logic and is entirely independent of the platform we're targeting.
    app.add_systems(Startup, spawn_player.after(load_sprites))
        .add_systems(Update, log_player_position)
        .add_systems(
            FixedUpdate,
            (
                control_player,
                flip_player_sprite,
                apply_gravity,
                apply_friction,
                apply_velocity,
                clamp_player_to_screen,
                reset_jumps,
            )
                .chain(),
        )
        .run();

    // Finally, we ensure this function never returns by entering an infinite loop if our app
    // ever exits.
    loop {}
}

fn setup_video(mut video: ResMut<Video>) {
    let (_background, mut vram) = video.0.tiled0();

    vram.set_background_palettes(&[Palette16::new([u16::MAX; 16])]);
}

fn load_sprites(mut loader: NonSendMut<SpriteLoader>, mut sprites: NonSendMut<Option<Sprites>>) {
    static SPRITES: &agb::display::object::Graphics =
        agb::include_aseprite!("assets/hero.aseprite");

    static SAW: &agb::display::object::Sprite = SPRITES.tags().get("Hero").sprite(0);

    let player = Sprite {
        handle: loader.get_vram_sprite(SAW),
        horizontal_flipped: false,
        vertical_flipped: false,
    };

    *sprites = Some(Sprites { player });
}

struct Sprites {
    player: Sprite,
}

fn log_player_position(player: Single<&Transform, With<Player>>) {
    info!(
        "Player: ({}, {})",
        player.translation.x, player.translation.y
    );
}

#[derive(Component)]
#[require(Gravity, Jumps, Velocity, Transform)]
struct Player;

#[derive(Component, Default)]
#[require(Velocity)]
struct Gravity;

#[derive(Component, Default)]
#[require(Transform)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Component, Default)]
struct Jumps {
    current: u8,
    max: u8,
}

fn spawn_player(mut commands: Commands, sprites: NonSend<Option<Sprites>>) {
    let sprites = sprites.as_ref().unwrap();
    commands.spawn((
        Transform::from_xyz(112., 144., 0.),
        sprites.player.clone(),
        Player,
        Jumps {
            max: 2,
            ..default()
        },
    ));
}

fn reset_jumps(mut query: Query<(&mut Jumps, &Transform)>) {
    for (mut jumps, transform) in &mut query {
        if transform.translation.y == 144. {
            jumps.current = 0;
        }
    }
}

fn flip_player_sprite(mut player: Single<&mut Sprite, With<Player>>, gamepad: Single<&Gamepad>) {
    if gamepad.just_pressed(GamepadButton::DPadLeft) {
        player.horizontal_flipped = true;
    }

    if gamepad.just_pressed(GamepadButton::DPadRight) {
        player.horizontal_flipped = false;
    }
}

fn control_player(
    player: Single<(&mut Velocity, &mut Jumps), With<Player>>,
    gamepad: Single<&Gamepad>,
) {
    let (mut velocity, mut jumps) = player.into_inner();

    if gamepad.pressed(GamepadButton::DPadLeft) {
        velocity.x -= 1.;
    }

    if gamepad.pressed(GamepadButton::DPadRight) {
        velocity.x += 1.;
    }

    if gamepad.just_pressed(GamepadButton::South) && jumps.current < jumps.max {
        jumps.current += 1;
        velocity.y = -5.;
    }

    velocity.x = velocity.x.clamp(-2., 2.);
}

fn apply_gravity(mut entities: Query<&mut Velocity, With<Gravity>>) {
    for mut velocity in &mut entities {
        velocity.y = (velocity.y + 0.4).clamp(-20., 20.);
    }
}

fn apply_friction(mut entities: Query<&mut Velocity>) {
    for mut velocity in &mut entities {
        velocity.x = velocity.x / 2.
    }
}

fn apply_velocity(mut entities: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut entities {
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
    }
}

fn clamp_player_to_screen(mut player: Single<&mut Transform, With<Player>>) {
    player.translation.x = player.translation.x.clamp(0., 224.);
    player.translation.y = player.translation.y.clamp(0., 144.);
}
