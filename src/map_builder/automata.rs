use crate::prelude::*;
use super::MapArchitect;

pub struct CellularAutomataArchitect {}

// 元胞自动机：地图上每个图块都独立地根据相邻图块的数量来决定是墙壁还是空地，不停地运行迭代，直至得到可用的地图。
impl MapArchitect for CellularAutomataArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder {
        let mut mb = MapBuilder {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            theme: super::themes::DungeonTheme::new(),
        };
        // 随机生成石墙和空地
        self.random_noise_map(rng, &mut mb.map);
        // 根据规则自动生成石墙和空地
        for _ in 0..10 {
            self.iteration(&mut mb.map);
        }
        // 找到距离地图中心最近的空地，以便放置玩家角色
        let start = self.find_start(&mb.map);
        // 50个怪物的位置坐标点
        mb.monster_spawns = mb.spawn_monsters(&start, rng);
        mb.player_start = start;
        // 放置护身符
        mb.amulet_start = mb.find_most_distant();
        mb
    }
}

impl CellularAutomataArchitect {
    fn random_noise_map(&mut self, rng: &mut RandomNumberGenerator, map: &mut Map) {
        // 随机生成墙壁和地板
        map.tiles.iter_mut().for_each(|t| {
            let roll = rng.range(0, 100);
            if roll > 55 {
                *t = TileType::Floor;
            } else {
                *t = TileType::Wall;
            }
        });
    }

    fn count_neighbors(&self, x: i32, y: i32, map: &Map) -> usize {
        let mut neighbors = 0;
        for iy in -1 ..= 1 {
            for ix in -1 ..= 1 {
                // 检查每一个相邻的图块，如果邻居是墙壁，则计数累加1。
                if !(ix==0 && iy == 0) && map.tiles[map_idx(x+ix, y+iy)] == TileType::Wall {
                    neighbors += 1;
                }
            }
        }
        neighbors
    }

    // 根据规则自动生成石墙和空地
    fn iteration(&mut self, map: &mut Map) {
        let mut new_tiles = map.tiles.clone();
        for y in 1 .. SCREEN_HEIGHT -1 {
            for x in 1 .. SCREEN_WIDTH -1 {
                // 获得邻居为墙壁的图块个数
                let neighbors = self.count_neighbors(x, y, map);
                let idx = map_idx(x, y);

                if neighbors > 4 || neighbors == 0 {
                    // 如果有0个或多于4个墙壁，则当前图块是墙壁
                    new_tiles[idx] = TileType::Wall;
                } else {
                    // 否则，当前图块是空地
                    new_tiles[idx] = TileType::Floor;
                }
            }
        }
        map.tiles = new_tiles;
    }

    // 找到距离地图中心最近的空地
    fn find_start(&self, map: &Map) -> Point {
        let center = Point::new(SCREEN_WIDTH/2, SCREEN_HEIGHT/2);
        let closest_point = map.tiles
            .iter()
            .enumerate() // 将迭代结果变成(index, tiletype)元组
            .filter(|(_, t)| **t == TileType::Floor)
            .map(|(idx, _)| (idx, DistanceAlg::Pythagoras.distance2d(
                                                                     center,
                                                                     map.index_to_point2d(idx)
            ))) // 计算每一个图块的距离
            .min_by(|(_, distance), (_, distance2)|
                        distance.partial_cmp(&distance2).unwrap()
            ) // 对图块进行排序，找到距离最近的空地
            .map(|(idx, _)| idx)
            .unwrap();
        map.index_to_point2d(closest_point)
    }
}