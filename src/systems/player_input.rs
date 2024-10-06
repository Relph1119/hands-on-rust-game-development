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
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(Weapon)]
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
            VirtualKeyCode::G => {
                // 获取玩家角色和玩家所在的位置
                let (player, player_pos) = players.iter(ecs).find_map(|(entity, pos)| Some((*entity, *pos))).unwrap();
                let mut items = <(Entity, &Item, &Point)>::query();
                // 捡起物品，将物品从Point组件中移除，添加到Carried组件中
                items.iter(ecs).filter(|(_entity, _item, &item_pos)| item_pos == player_pos)
                    .for_each(|(entity, _item, _item_pos)| {
                        commands.remove_component::<Point>(*entity);
                        commands.add_component(*entity, Carried(player));

                        if let Ok(e) = ecs.entry_ref(*entity) {
                            // 检查物品是不是武器
                            if e.get_component::<Weapon>().is_ok() {
                                <(Entity, &Carried, &Weapon)>::query()
                                    .iter(ecs)
                                    .filter(|(_, c, _)| c.0 == player)
                                    .for_each(|(e, _c, _w)| {
                                        // 如果是武器，则从游戏世界中移除
                                        commands.remove(*e);
                                    })
                            }
                        }
                    });
                Point::new(0, 0)
            },
            // 使用物品
            VirtualKeyCode::Key1 => use_item(0, ecs, commands),
            VirtualKeyCode::Key2 => use_item(1, ecs, commands),
            VirtualKeyCode::Key3 => use_item(2, ecs, commands),
            VirtualKeyCode::Key4 => use_item(3, ecs, commands),
            VirtualKeyCode::Key5 => use_item(4, ecs, commands),
            VirtualKeyCode::Key6 => use_item(5, ecs, commands),
            VirtualKeyCode::Key7 => use_item(6, ecs, commands),
            VirtualKeyCode::Key8 => use_item(7, ecs, commands),
            VirtualKeyCode::Key9 => use_item(8, ecs, commands),
            _ => Point::new(0, 0),
        };
        // 获取目标点和玩家角色实体
        let (player_entity, destination) = players
            .iter(ecs)
            .find_map(|(entity, pos)| Some((*entity, *pos + delta)))
            .unwrap();

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
                    commands.push(((), WantsToAttack {
                        attacker: player_entity,
                        victim: *entity,
                    }));
                });
            // 如果没有碰到任何东西
            if !hit_something {
                commands.push(((), WantsToMove {
                    entity: player_entity,
                    destination,
                }));
            }
        }

        *turn_state = TurnState::PlayerTurn;
    }
}

fn use_item(n: usize, ecs: &mut SubWorld, commands: &mut CommandBuffer) -> Point {
    // 获得玩家角色实体
    let player_entity = <(Entity, &Player)>::query().iter(ecs)
        .find_map(|(entity, _player)| Some(*entity))
        .unwrap();

    // 过滤掉枚举计数值不等于n的物品，并获取第1个物品实体
    let item_entity = <(Entity, &Item, &Carried)>::query().iter(ecs)
        .filter(|(_, _, carried)| carried.0 == player_entity)
        .enumerate()
        .filter(|(item_count, (_, _, _))| *item_count == n)
        .find_map(|(_, (item_entity, _, _))| Some(*item_entity));


    if let Some(item_entity) = item_entity {
        // 如果物品存在
        commands.push(((), ActivateItem{
                used_by: player_entity,
                item: item_entity
            }));
    }

    Point::zero()
}
