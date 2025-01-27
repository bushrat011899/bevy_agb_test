use alloc::boxed::Box;
use bevy_app::{Plugin, Startup};
use bevy_ecs::{resource::Resource, system::ResMut};
use bevy_platform_support::{sync::atomic::AtomicU32, time::Instant};
use core::{sync::atomic::Ordering, time::Duration};

#[derive(Default)]
pub struct AgbTimePlugin;

impl Plugin for AgbTimePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        static TIMER_2_INTERRUPTS: AtomicU32 = AtomicU32::new(0);

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

        app.add_systems(Startup, start_timer_2);
    }
}

pub fn start_timer_2(mut timer: ResMut<Timer<2>>) {
    timer
        .0
        .set_enabled(true)
        .set_divider(agb::timer::Divider::Divider1)
        .set_overflow_amount(0xFFFF)
        .set_interrupt(true);
}

/// Provides a single timer.
#[derive(Resource)]
pub struct Timer<const ID: u8>(pub agb::timer::Timer);
