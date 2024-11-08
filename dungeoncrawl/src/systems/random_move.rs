use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(MovingRandomly)]
#[read_component(Health)]
#[read_component(Player)]
pub fn random_move(ecs: &SubWorld, commands: &mut CommandBuffer) {
    let mut movers = <(Entity, &Point, &MovingRandomly)>::query();
    let mut positions = <(Entity, &Point, &Health)>::query();
    movers.iter(ecs).for_each(|(entity, pos, _)| {
        let mut rng = RandomNumberGenerator::new();
        let destination = match rng.range(0, 4) {
            0 => Point::new(-1, 0),
            1 => Point::new(1, 0),
            2 => Point::new(0, -1),
            _ => Point::new(0, 1),
        } + *pos;

        let mut attacked = false;
        // 先查询实体，使用过滤器筛选出位于目标图块之上的实体
        positions
            .iter(ecs)
            .filter(|(_, target_pos, _)| **target_pos == destination)
            .for_each(|(victim, _, _)| {
                // 如果目标位置上有实体，检查这个是替是否有Player组件
                if ecs.entry_ref(*victim)
                    .unwrap().get_component::<Player>().is_ok()
                {
                    // 发出攻击命令
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
    });
}