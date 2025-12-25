use bevy::{
    math::{Vec2, Vec3Swizzles},
    transform::components::Transform,
};

use crate::{Physics, GRAVITY_STRENGTH};

#[allow(dead_code)]
pub fn apply_gravity_toward_normal(physics: &mut Physics, falling: bool) {
    if falling {
        physics.acceleration.y = -GRAVITY_STRENGTH;
    } else {
        let gravity_normal_dir = physics.normal * GRAVITY_STRENGTH;
        physics.acceleration += gravity_normal_dir;
    }
}

#[allow(dead_code)]
pub fn update_physics_and_transform(physics: &mut Physics, transform: &mut Transform) {
    // Update velocity
    let new_velocity = physics.velocity + physics.acceleration;
    physics.velocity = new_velocity;

    // Update previous position
    physics.prev_position = transform.translation.xy();
    // Update position
    transform.translation.x += physics.velocity.x;
    transform.translation.y += physics.velocity.y;
}

#[allow(dead_code)]
pub fn apply_movement_acceleration(
    physics: &mut Physics,
    move_dir: &Vec2,
    falling: bool,
    no_move_dir: bool,
    max_speed: f32,
    acceleration_scalers: (f32, f32),
) {
    // If the agent is falling
    if falling {
        physics.acceleration = Vec2::ZERO;
        return;
    }

    // Apply acceleration
    physics.acceleration = (*move_dir * max_speed - physics.velocity)
        * if no_move_dir {
            // Deacceleration
            acceleration_scalers.1
        } else {
            // Acceleration
            acceleration_scalers.0
        };
}

#[allow(dead_code)]
pub fn handle_jumping(physics: &mut Physics, falling: bool, jump_velocity: Vec2) {
    // If the agent is trying to jump
    if jump_velocity.length_squared() > 0.0 && !falling {
        // If on the ground
        if physics.grounded {
            // Jump
            physics.velocity = jump_velocity;
            physics.acceleration.x = 0.0;
            physics.acceleration.y = -GRAVITY_STRENGTH;
            physics.grounded = false;
            physics.has_wall_jumped = false;
            physics.walled = 0;

            println!("Jump!!!");
            println!("Initial Jump Velocity: {}", jump_velocity.length());
        }
        // If on a wall
        else if physics.walled != 0 {
            // Wall jump
            physics.velocity = jump_velocity;
            physics.acceleration.x = 0.0;
            physics.acceleration.y = -GRAVITY_STRENGTH;
            physics.walled = 0;
            physics.grounded = false;
            physics.has_wall_jumped = true;

            println!("Wall Jump!!!");
            println!("Initial Jump Velocity: {}", jump_velocity.length());
        }
    }
}
