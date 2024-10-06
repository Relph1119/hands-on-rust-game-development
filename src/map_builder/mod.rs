mod empty;
mod rooms;
mod automata;
mod drunkard;
mod prefab;
mod themes;

use crate::prelude::*;

use rooms::RoomsArchitect;
use crate::map_builder::automata::CellularAutomataArchitect;
use crate::map_builder::drunkard::DrunkardsWalkArchitect;
use crate::map_builder::prefab::apply_prefab;
use crate::map_builder::themes::{DungeonTheme, ForestTheme};

trait MapArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder;
}

// 定义20个房间的地下城
const NUM_ROOMS: usize = 20;

pub struct MapBuilder {
    pub map: Map,
    // 将要被添加到地图中的房间
    pub rooms: Vec<Rect>,
    // 怪物列表
    pub monster_spawns: Vec<Point>,
    // 玩家的初始位置
    pub player_start: Point,
    // 护身符的位置
    pub amulet_start: Point,
    // 主题风格
    pub theme: Box<dyn MapTheme>
}

impl MapBuilder {
    pub fn new(rng: &mut RandomNumberGenerator) -> Self {
        // 装箱操作，dyn表示动态分发
        let mut architect : Box<dyn MapArchitect> = match rng.range(0,3) {
            0 => Box::new(DrunkardsWalkArchitect{}),
            1 => Box::new(RoomsArchitect{}),
            _ => Box::new(CellularAutomataArchitect{})
        };
        let mut mb = architect.new(rng);
        // 放置金库
        apply_prefab(&mut mb, rng);

        mb.theme = match rng.range(0, 2) {
            0 => DungeonTheme::new(),
            _ => ForestTheme::new()
        };
        mb
    }

    fn fill(&mut self, tile: TileType) {
        // 填充石墙
        self.map.tiles.iter_mut().for_each(|t| *t = tile);
    }

    fn find_most_distant(&self) -> Point {
        let dijkstra_map = DijkstraMap::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &vec![self.map.point2d_to_index(self.player_start)],
            &self.map,
            1024.0,
        );
        const UNREACHABLE: &f32 = &f32::MAX;
        // 找到距离玩家最远的，且可到达的图块，将其设置为护身符的位置
        self.map.index_to_point2d(
            dijkstra_map.map
                .iter()
                .enumerate()
                .filter(|(_, dist)| *dist < UNREACHABLE)
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap().0
        )
    }

    fn build_random_rooms(&mut self, rng: &mut RandomNumberGenerator) {
        while self.rooms.len() < NUM_ROOMS {
            let room = Rect::with_size(
                rng.range(1, SCREEN_WIDTH - 10),
                rng.range(1, SCREEN_HEIGHT - 10),
                rng.range(2, 10),
                rng.range(2, 10),
            );
            // 判断是否重叠
            let mut overlap = false;
            for r in self.rooms.iter() {
                if r.intersect(&room) {
                    overlap = true;
                }
            }
            // 填充地板，开凿房间
            if !overlap {
                room.for_each(|p| {
                    if p.x > 0 && p.x < SCREEN_WIDTH && p.y > 0 && p.y < SCREEN_HEIGHT {
                        let idx = map_idx(p.x, p.y);
                        self.map.tiles[idx] = TileType::Floor;
                    }
                });
                self.rooms.push(room)
            }
        }
    }

    // 生成垂直方向的通道
    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        use std::cmp::{min, max};
        for y in min(y1, y2)..=max(y1, y2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx] = TileType::Floor;
            }
        }
    }

    // 生成水平方向的通道
    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        use std::cmp::{max, min};
        for x in min(x1, x2)..=max(x1, x2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx] = TileType::Floor;
            }
        }
    }

    // 生成房间之间的完整走廊
    fn build_corridors(&mut self, rng: &mut RandomNumberGenerator) {
        let mut rooms = self.rooms.clone();
        // 按照各个房间的中心点的位置对房间进行排序，避免出现连接两个较远房间的蛇形走廊
        rooms.sort_by(|a, b| a.center().x.cmp(&b.center().x));

        for (i, room) in rooms.iter().enumerate().skip(1) {
            let prev = rooms[i - 1].center();
            let new = room.center();

            // 随机选择一种开凿走廊的方式，先水平后垂直，或者先垂直后水平
            if rng.range(0, 2) == 1 {
                self.apply_horizontal_tunnel(prev.x, new.x, prev.y);
                self.apply_vertical_tunnel(prev.y, new.y, new.x);
            } else {
                self.apply_vertical_tunnel(prev.y, new.y, prev.x);
                self.apply_horizontal_tunnel(prev.x, new.x, new.y);
            }
        }
    }

    // 选出能够放置50个怪物的位置
    fn spawn_monsters(&self, start: &Point, rng: &mut RandomNumberGenerator) -> Vec<Point> {
        // 怪物数量为50个
        const NUM_MONSTERS : usize = 50;
        // 过滤出空地并且与玩家起始位置距离大于10的图块
        let mut spawnable_tiles : Vec<Point> = self.map.tiles
            .iter()
            .enumerate()
            .filter(|(idx, t)|
                **t == TileType::Floor &&
                    DistanceAlg::Pythagoras.distance2d(
                        *start,
                        self.map.index_to_point2d(*idx)
                    ) > 10.0
            )
            .map(|(idx, _)| self.map.index_to_point2d(idx))
            .collect();

        let mut spawns = Vec::new();
        for _ in 0 .. NUM_MONSTERS {
            // 生成一个怪物的出生点坐标编号
            let target_index = rng.random_slice_index(&spawnable_tiles).unwrap();
            spawns.push(spawnable_tiles[target_index].clone());
            // 已选过的图块则从待选区移除
            spawnable_tiles.remove(target_index);
        }
        spawns
    }
}

/*
 * Sync：可以从不同的线程安全访问
 * Send：可以在不同的线程之间传递、转移变量
 */
pub trait MapTheme: Sync + Send {
    fn tile_to_render(&self, tile_type: TileType) -> FontCharType;
}