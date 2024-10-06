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
            },
            Health{current: 10, max: 10}
        )
    );
}

pub fn spawn_monster(
    ecs: &mut World,
    rng: &mut RandomNumberGenerator,
    pos: Point
) {
    /* 随机从4种怪物中选择一个，其中：
     * 1..8产生妖精，否则产生兽人。
     */
    let (hp, name, glyph) = match rng.roll_dice(1, 10) {
        1..=8 => goblin(),
        _ => orc()
    };

    ecs.push(
        (
            Enemy,
            pos,
            Render {
                color: ColorPair::new(WHITE, BLACK),
                glyph,
            },
            ChasingPlayer{},
            Health{current: hp, max: hp},
            Name(name)
        )
    );
}

fn goblin() -> (i32, String, FontCharType) {
    (1, "Goblin".to_string(), to_cp437('g'))
}

fn orc() -> (i32, String, FontCharType) {
    (2, "Orc".to_string(), to_cp437('o'))
}

pub fn spawn_amulet_of_yala(ecs: &mut World, pos: Point) {
    ecs.push((
        Item, AmuletOfYala, pos,
        Render {
            color: ColorPair::new(WHITE, BLACK),
            glyph: to_cp437('|')
        },
        Name("Amulet of Yala".to_string())
    ));
}