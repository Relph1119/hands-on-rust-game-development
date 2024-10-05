use crate::prelude::*;

/* 碰撞检测
 * CommandBuffer：在系统的逻辑执行完成以后，再去执行里面的指令
 */
#[system]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(Enemy)]
pub fn collisions(ecs: &mut SubWorld, commands: &mut CommandBuffer) {
    // 查询玩家位置
    let mut player_pos = Point::zero();
    let mut players = <&Point>::query().filter(component::<Player>());
    players.iter(ecs).for_each(|pos| player_pos = *pos);
    // 查询怪物位置
    let mut enemies = <(Entity, &Point)>::query().filter(component::<Enemy>());
    // 如果玩家角色和任何一个敌人发生碰撞，则移除掉这个敌人
    let mut enemies = <(Entity, &Point)>::query().filter(component::<Enemy>());
    enemies.iter(ecs)
        .filter(|(_,pos)| **pos == player_pos)
        .for_each(|(entity, _)| commands.remove(*entity));
}