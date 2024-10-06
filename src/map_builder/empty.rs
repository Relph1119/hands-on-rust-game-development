use crate::prelude::*;
use super::MapArchitect;

pub struct EmptyArchitect {}

impl MapArchitect for EmptyArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder {
        let mut mb = MapBuilder {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            theme: super::themes::DungeonTheme::new(),
        };
        // 填充地板
        mb.fill(TileType::Floor);
        // 玩家位于整个地图中央
        mb.player_start = Point::new(SCREEN_WIDTH/ 2, SCREEN_HEIGHT / 2);
        // 放置亚拉的护身符
        mb.amulet_start = mb.find_most_distant();
        for _ in 0..50 {
            // 添加怪物
            mb.monster_spawns.push(
                Point::new(
                    rng.range(1, SCREEN_WIDTH),
                    rng.range(1, SCREEN_HEIGHT)
                )
            )
        }
        mb
    }
}