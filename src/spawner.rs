use crate::prelude::*;

pub fn spawn_player(ecs: &mut World, pos: Point) {
    // 将多个组件聚合在一个实体中，由玩家、位置信息、渲染组件构成。
    ecs.push(
        (
            Player,
            pos,
            Render{
                color: ColorPair::new(WHITE, BLACK),
                glyph: to_cp437('@')
            }
        )
    );
}

pub fn spawn_monster(
    ecs: &mut World,
    rng: &mut RandomNumberGenerator,
    pos: Point
) {
    /* 随机从4种怪物中选择一个，其中：
     * E代表双头怪，O代表食人魔，o代表兽人，g代表妖精。
     */
    ecs.push(
        (
            Enemy,
            pos,
            Render {
                color: ColorPair::new(WHITE, BLACK),
                glyph: match rng.range(0, 4) {
                    0 => to_cp437('E'),
                    1 => to_cp437('O'),
                    2 => to_cp437('o'),
                    _ => to_cp437('g'),
                }
            }
        )
    );
}