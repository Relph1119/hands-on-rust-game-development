use crate::prelude::*;
#[system]
#[read_component(ActivateItem)]
#[read_component(ProvidesHealing)]
#[write_component(Health)]
#[read_component(ProvidesDungeonMap)]
pub fn use_items(ecs: &mut SubWorld,
                 commands: &mut CommandBuffer,
                 #[resource] map: &mut Map) {
    /*
     * Rust借用的硬性规定：
     * 1. 可以对一个变量进行任意多次的不可变借用。
     * 2. 同一时刻只能对一个变量进行一次可变借用。
     * 3. 不能同时以可变和不可变的形式借用一个变量。
     */
    let mut healing_to_apply = Vec::<(Entity, i32)>::new();

    <(Entity, &ActivateItem)>::query().iter(ecs)
        .for_each(|(entity, activate)| {
            // 获取物品实体
            let item = ecs.entry_ref(activate.item);
            if let Ok(item) = item {
                if let Ok(healing) = item.get_component::<ProvidesHealing>() {
                    // 如果是治疗药水，则加入到向量中
                    healing_to_apply.push((activate.used_by, healing.amount));
                }

                if let Ok(_mapper) = item.get_component::<ProvidesDungeonMap>() {
                    // 如果是地图，则把所有地块都展示出来
                    map.revealed_tiles.iter_mut().for_each(|t| *t = true);
                }
            }

            commands.remove(activate.item);
            commands.remove(*entity);
        });

    // 执行疗伤
    for heal in healing_to_apply.iter() {
        if let Ok(mut target) = ecs.entry_mut(heal.0) {
            // 获取生命值
            if let Ok(health) = target.get_component_mut::<Health>() {
                // 恢复生命
                health.current = i32::min(
                    health.max,
                    health.current + heal.1,
                );
            }
        }
    }
}