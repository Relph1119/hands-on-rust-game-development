use crate::prelude::*;

/* system：将函数名变为player_input_system，并使用一些Legion构建系统时所需要的额外代码将这个函数包装起来。
 * write_component：为Point组件类型申请可写入权限。
 * read_component：为Player组件类型申请可读权限。
 */
#[system]
#[write_component(Point)]
#[read_component(Player)]
pub fn player_input(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] key: &Option<VirtualKeyCode>,
    #[resource] turn_state: &mut TurnState
) {
    let mut players = <(Entity, &Point)>::query()
        .filter(component::<Player>());
    if let Some(key) = *key {
        let delta = match key {
            VirtualKeyCode::Left => Point::new(-1, 0),
            VirtualKeyCode::Right => Point::new(1, 0),
            VirtualKeyCode::Up => Point::new(0, -1),
            VirtualKeyCode::Down => Point::new(0, 1),
            _ => Point::new(0, 0),
        };
        players.iter(ecs).for_each(|(entity, pos)| {
            let destination = *pos + delta;
            commands.push(((), WantsToMove{entity: *entity, destination}));
            // 当玩家做出操作后，状态变更为PlayerTurn
            *turn_state = TurnState::PlayerTurn;
        });
    }
}
