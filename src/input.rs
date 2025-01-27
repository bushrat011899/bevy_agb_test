use alloc::string::ToString;
use bevy_app::{First, Plugin, Startup};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::EventWriter,
    query::With,
    resource::Resource,
    system::{ResMut, Single},
};
use bevy_input::gamepad::{
    GamepadButton, GamepadConnection, GamepadConnectionEvent, RawGamepadButtonChangedEvent,
    RawGamepadEvent,
};

#[derive(Default)]
pub struct AgbInputPlugin;

impl Plugin for AgbInputPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.world_mut().spawn(GameBoyGamepad);

        app.insert_resource(ButtonController::new())
            .add_systems(Startup, connect_gamepad)
            .add_systems(First, update_gamepad);
    }
}

/// Helper to make it easy to get the current state of the GBA's buttons.
#[derive(Resource)]
pub struct ButtonController(pub agb::input::ButtonController);

impl ButtonController {
    #[must_use]
    pub fn new() -> Self {
        Self(agb::input::ButtonController::new())
    }
}

#[derive(Component)]
pub struct GameBoyGamepad;

pub fn connect_gamepad(
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

pub fn update_gamepad(
    mut manager: ResMut<ButtonController>,
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
