use crate::prelude::*;

#[system]
#[read_component(WantsToAttack)]
#[read_component(Player)]
#[write_component(Health)]
#[read_component(Damage)]
#[read_component(Carried)]
pub fn combat(ecs: &mut SubWorld, commands: &mut CommandBuffer) {
    // 希望发起攻击的实体列表
    let mut attackers = <(Entity, &WantsToAttack)>::query();
    // 被攻击者的列表，根据攻击者信息来计算它们产生的破坏力输出
    let victims: Vec<(Entity, Entity, Entity)> = attackers
        .iter(ecs)
        .map(|(entity, attack)| (*entity, attack.attacker, attack.victim))
        .collect();
    victims.iter().for_each(|(message, attacker, victim)| {
        // 获取玩家角色
        let is_player = ecs.entry_ref(*victim).unwrap().get_component::<Player>().is_ok();
        // 获得攻击者的基础伤害值
        let base_damage = if let Ok(v) = ecs.entry_ref(*attacker) {
            if let Ok(dmg) = v.get_component::<Damage>() {
                dmg.0
            } else {
                0
            }
        } else {
            0
        };
        // 获得携带武器的伤害值总和
        let weapon_damage: i32 = <(&Carried, &Damage)>::query().iter(ecs)
            .filter(|(carried, _)| carried.0 == *attacker)
            .map(|(_, dmg)| dmg.0)
            .sum();
        // 计算最终伤害值
        let final_damage = base_damage + weapon_damage;

        // 针对只包含生命值的被攻击对象执行操作
        if let Ok(health) = ecs
            .entry_mut(*victim)
            .unwrap()
            .get_component_mut::<Health>()
        {
            // println!("Health before attack: {}", health.current);
            health.current -= final_damage;
            // 消灭怪物
            if health.current < 1 && !is_player {
                commands.remove(*victim);
            }
            // println!("Health after attack: {}", health.current);
        }
        commands.remove(*message);
    });
}