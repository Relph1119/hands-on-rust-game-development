use crate::prelude::*;
mod player_input;
mod map_render;
mod entity_render;
mod collisions;

pub fn build_scheduler() -> Schedule {
    // 创建游戏中各个系统的执行计划
    Schedule::builder()
        .add_system(player_input::player_input_system())
        .add_system(collisions::collisions_system())
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .build()
}