use crate::prelude::*;

/* system：将函数名变为player_input_system，并使用一些Legion构建系统时所需要的额外代码将这个函数包装起来。
 * write_component：为Point组件类型申请可写入权限。
 * read_component：为Player组件类型申请可读权限。
 */
#[system]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(Enemy)]
#[write_component(Health)]
pub fn player_input(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] key: &Option<VirtualKeyCode>,
    #[resource] turn_state: &mut TurnState,
) {
    let mut players = <(Entity, &Point)>::query().filter(component::<Player>());
    let mut enemies = <(Entity, &Point)>::query().filter(component::<Enemy>());

    if let Some(key) = *key {
        let delta = match key {
            VirtualKeyCode::Left => Point::new(-1, 0),
            VirtualKeyCode::Right => Point::new(1, 0),
            VirtualKeyCode::Up => Point::new(0, -1),
            VirtualKeyCode::Down => Point::new(0, 1),
            _ => Point::new(0, 0),
        };
        // 获取目标点和玩家角色实体
        let (player_entity, destination) = players
            .iter(ecs)
            .find_map(|(entity, pos)| Some((*entity, *pos + delta)))
            .unwrap();

        // 是否执行了某些操作
        let mut did_something = false;
        // 如果位置有移动
        if delta.x != 0 || delta.y != 0 {
            // 是否发生战斗的标志
            let mut hit_something = false;
            // 如果有匹配的实体，则运行闭包，否则，空的迭代器会直接跳过这一步
            enemies
                .iter(ecs)
                .filter(|(_, pos)| {
                    **pos == destination
                })
                .for_each(|(entity, _)| {
                    // 玩家角色正在面对一个怪物
                    hit_something = true;
                    did_something = true;
                    commands.push(((), WantsToAttack {
                        attacker: player_entity,
                        victim: *entity,
                    }));
                });
            // 如果没有碰到任何东西
            if !hit_something {
                did_something = true;
                commands.push(((), WantsToMove {
                    entity: player_entity,
                    destination,
                }));
            }
        }

        if !did_something {
            // 如果没有执行操作，自愿等待，可以获得治疗
            if let Ok(mut health) = ecs
                .entry_mut(player_entity)
                .unwrap()
                .get_component_mut::<Health>() {
                health.current = i32::min(health.max, health.current + 1);
            }
        }
        *turn_state = TurnState::PlayerTurn;
    }
}
