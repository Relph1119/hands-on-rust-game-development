use crate::prelude::*;
use super::MapArchitect;

/* Drunkard's Walk算法：
 * 在石墙遍布的地图中随机安放一个醉酒的矿工，矿工会随机挖掘，在地图上开凿出路径。
 * 如果矿工达到行走的最大步数限制，或者走出了地图，则停止。
 * 如果地图还没有生成好，那么就继续生成一个醉酒的矿工。直到满足空地图块的数量要求。
 */
pub struct DrunkardsWalkArchitect {}

// 设置矿工能行走的最大步数
const STAGGER_DISTANCE: usize = 400;

const NUM_TILES: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;
// 空地图块必须占全图的30%
const DESIRED_FLOOR: usize = NUM_TILES / 3;

impl MapArchitect for DrunkardsWalkArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder {
        let mut mb = MapBuilder {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            theme: super::themes::DungeonTheme::new(),
        };
        // 先全部填充石墙
        mb.fill(TileType::Wall);
        let center = Point::new(SCREEN_WIDTH /2, SCREEN_HEIGHT/2);
        // 矿工开始挖掘
        self.drunkard(&center, rng, &mut mb.map);
        // 循环判断是否继续需要矿工挖掘
        while mb.map.tiles.iter().filter(|t| **t == TileType::Floor).count() < DESIRED_FLOOR {
            // 随机在一个点上放置矿工挖掘
            self.drunkard(
                &Point::new(
                    rng.range(0, SCREEN_WIDTH),
                    rng.range(0, SCREEN_HEIGHT)
                ),
                rng,
                &mut mb.map
            );
            let dijkstra_map = DijkstraMap::new(
                                                SCREEN_WIDTH,
                                                SCREEN_HEIGHT,
                                                &vec![mb.map.point2d_to_index(center)],
                                                &mb.map,
                                                1024.0);
            // 将不可达的点设置为石墙
            dijkstra_map.map.iter().enumerate()
                .filter(|(_, distance)| *distance > &2000.0)
                .for_each(|(idx, _)| mb.map.tiles[idx] = TileType::Wall);
        }
        mb.monster_spawns = mb.spawn_monsters(&center, rng);
        mb.player_start = center;
        mb.amulet_start = mb.find_most_distant();
        mb
    }


}

impl DrunkardsWalkArchitect {
    fn drunkard(&mut self, start: &Point, rng: &mut RandomNumberGenerator, map: &mut Map) {
        // 矿工的位置
        let mut drunkard_pos = start.clone();
        // 记录已经走过的步数
        let mut distance_staggered = 0;

        // 使用loop会不停循环运行，直至遇到break。
        loop {
            let drunk_idx = map.point2d_to_index(drunkard_pos);
            map.tiles[drunk_idx] = TileType::Floor;

            // 随机向4个方向行走
            match rng.range(0, 4) {
                0 => drunkard_pos.x -= 1,
                1 => drunkard_pos.x += 1,
                2 => drunkard_pos.y -= 1,
                _ => drunkard_pos.y += 1,
            }

            if !map.in_bounds(drunkard_pos) {
                // 如果走出边界，则停止
                break;
            }

            distance_staggered += 1;
            if distance_staggered > STAGGER_DISTANCE {
                // 如果达到最大步数，则停止
                break;
            }
        }
    }
}