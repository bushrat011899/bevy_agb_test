use bevy_app::{Last, Plugin};
use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{NonSendMut, Query},
};
use bevy_math::Vec3;
use bevy_transform::components::GlobalTransform;

#[derive(Default)]
pub struct AgbRenderPlugin;

impl Plugin for AgbRenderPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(Last, render_objects);
    }
}

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

#[derive(Resource)]
pub struct Video(pub agb::display::video::Video);

#[derive(Resource)]
pub struct WindowDist(pub agb::display::WindowDist);

#[derive(Resource)]
pub struct BlendDist(pub agb::display::BlendDist);

/// Manages access to the Game Boy Advance's DMA
#[derive(Resource)]
pub struct DmaController(pub agb::dma::DmaController);
