pub mod movement;
pub mod wander;

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        schedule::IntoScheduleConfigs,
        system::{Query, Res},
    },
    transform::components::Transform,
};

use crate::{s_move_goal_point, Physics};

use super::pathfinding::PathfindingGraph;

pub const PURSUE_AI_AGENT_RADIUS: f32 = 8.0;

pub enum PursueAIState {
    Wander,
    Pursue,
    Search,
    Attack,
}

pub struct PursueAIPlugin;

impl Plugin for PursueAIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, s_pursue_ai_update.after(s_move_goal_point));
    }
}

#[derive(Component)]
pub struct PursueAI {
    pub state: PursueAIState,
}

pub fn s_pursue_ai_update(
    mut platformer_ai_query: Query<(&mut Transform, &mut Physics, &mut PursueAI)>,
    pathfinding: Res<PathfindingGraph>,
) {
    for (mut transform, mut physics, mut pursue_ai) in platformer_ai_query.iter_mut() {
        let next_state: Option<PursueAIState> = match pursue_ai.state {
            PursueAIState::Wander => wander::wander_update(
                &mut transform,
                &mut physics,
                &mut pursue_ai,
                pathfinding.as_ref(),
            ),
            // PursueAIState::Pursue => {}
            // PursueAIState::Search => {}
            // PursueAIState::Attack => {}
            _ => None,
        };

        if let Some(new_state) = next_state {
            pursue_ai.state = new_state;
        }
    }
}
