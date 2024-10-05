use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(ChasingPlayer)]
#[read_component(Health)]
#[read_component(Player)]
pub fn chasing(
    #[resource] map: &Map,
    ecs: &SubWorld,
    commands: &mut CommandBuffer,
) {
    let mut movers = <(Entity, &Point, &ChasingPlayer)>::query();
    let mut positions = <(Entity, &Point, &Health)>::query();
    let mut player = <(&Point, &Player)>::query();
    // 查询玩家所在的位置
    let player_pos = player.iter(ecs).nth(0).unwrap().0;
    // 玩家所在的图块索引编号
    let player_idx = map_idx(player_pos.x, player_pos.y);

    // 创建一个包含玩家角色当前坐标的向量，作为起始点
    let search_targets = vec![player_idx];
    // 初始化迪杰斯特拉图，1024表示在停止计算前可以走出的最远距离
    let dijkstra_map = DijkstraMap::new(
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        &search_targets,
        map,
        1024.0,
    );

    // 追杀玩家：如果怪物与玩家处于相邻位置，一定会攻击玩家，如果不相邻，怪物沿着迪杰斯特拉图的路线追击玩家。
    movers.iter(ecs).for_each(|(entity, pos, _)| {
        let idx = map_idx(pos.x, pos.y);
        // 找到最近的一个位置
        if let Some(destination) = DijkstraMap::find_lowest_exit(&dijkstra_map, idx, map) {
            // 计算怪物与玩家的距离
            let distance = DistanceAlg::Pythagoras.distance2d(*pos, *player_pos);
            // 使用1.2可以保证怪物不会在对角线位置上发起攻击
            let destination = if distance > 1.2 {
                map.index_to_point2d(destination)
            } else {
                *player_pos
            };

            // 怪物随机行走
            let mut attacked = false;
            positions.iter(ecs)
                .filter(|(_, target_pos, _)| **target_pos == destination)
                .for_each(|(victim, _, _)| {
                    if ecs.entry_ref(*victim).unwrap().get_component::<Player>().is_ok() {
                        commands.push(((), WantsToAttack {
                            attacker: *entity,
                            victim: *victim,
                        }));
                    }
                    attacked = true;
                });

            if !attacked {
                commands.push(((), WantsToMove { entity: *entity, destination }));
            }
        }
    });
}