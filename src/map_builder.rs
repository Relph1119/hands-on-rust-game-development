use crate::prelude::*;

// 定义20个房间的地下城
const NUM_ROOMS: usize = 20;

pub struct MapBuilder {
    pub map: Map,
    // 将要被添加到地图中的房间
    pub rooms: Vec<Rect>,
    // 玩家的初始位置
    pub player_start: Point,
    // 护身符的位置
    pub amulet_start: Point
}

impl MapBuilder {
    pub fn new(rng: &mut RandomNumberGenerator) -> Self {
        let mut mb = MapBuilder{
            map: Map::new(),
            rooms: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero()
        };
        // 先填充石墙
        mb.fill(TileType::Wall);
        // 开凿房间
        mb.build_random_rooms(rng);
        // 开凿走廊
        mb.build_corridors(rng);
        // 玩家从第1个房间的中央开始
        mb.player_start = mb.rooms[0].center();

        let dijkstra_map = DijkstraMap::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &vec![mb.map.point2d_to_index(mb.player_start)],
            &mb.map,
            1024.0
        );
        // 用于表示图块不可达
        const UNREACHABLE : &f32 = &f32::MAX;
        // 找到距离玩家最远的，且可到达的图块，将其设置为护身符的位置
        mb.amulet_start = mb.map.index_to_point2d(
            dijkstra_map.map
                .iter()
                .enumerate()
                .filter(|(_,dist)| *dist < UNREACHABLE)
                .max_by(|a,b| a.1.partial_cmp(b.1).unwrap())
                .unwrap().0
        );
        mb
    }

    fn fill(&mut self, tile: TileType) {
        // 填充石墙
        self.map.tiles.iter_mut().for_each(|t| *t = tile);
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
}