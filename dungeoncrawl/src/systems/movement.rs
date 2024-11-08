use crate::prelude::*;

/*
 * system(for_each)：表示为每一个匹配到的实体运行一次这个系统函数。
 */
#[system(for_each)]
#[read_component(Player)]
#[read_component(FieldOfView)]
pub fn movement(
    entity: &Entity,
    want_move: &WantsToMove,
    #[resource] map: &mut Map,
    #[resource] camera: &mut Camera,
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    if map.can_enter_tile(want_move.destination) {
        // 一次性快速批量执行这些更新。
        commands.add_component(want_move.entity, want_move.destination);
        // 用来表示这个实体在当前这个子世界中是否有效，只有在系统声明中read_component或write_component之后，这个实体才有效。
        if let Ok(entry) = ecs.entry_ref(want_move.entity) {
            if let Ok(fov) = entry.get_component::<FieldOfView>() {
                commands.add_component(want_move.entity, fov.clone_dirty());

                if entry.get_component::<Player>().is_ok() {
                    // 得到这个实体之后，更新与玩家角色相关摄像机的信息。
                    camera.on_player_move(want_move.destination);

                    // 对于处在玩家可见区域的每一个图块，将revealed_tiles都设置为true
                    fov.visible_tiles.iter().for_each(|pos| {
                        map.revealed_tiles[map_idx(pos.x, pos.y)] = true;
                    });
                }
            }
        }
    }
    // 删除处理过的信息，否则这些信息在下一次运行时还会被处理一次。
    commands.remove(*entity);
}