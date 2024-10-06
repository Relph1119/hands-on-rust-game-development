use crate::prelude::*;

const NUM_TILES: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

/*
 * Clone类型：添加一个clone()函数
 * Copy类型：不再转移变量的所有权，做一个拷贝
 * PartialEq类型：可以使用==比较两个TileType类型的变量
 */
#[derive(Copy, Clone, PartialEq)]
pub enum TileType {
    // 墙壁
    Wall,
    // 地板
    Floor,
}

pub struct Map {
    // 地块
    pub tiles: Vec<TileType>,
}

// 计算地图索引，行优先的编码方式
pub fn map_idx(x: i32, y: i32) -> usize {
    ((y * SCREEN_WIDTH) + x) as usize
}

impl Map {
    pub fn new() -> Self {
        Self {
            tiles: vec![TileType::Floor; NUM_TILES],
        }
    }

    // 判断玩家是否越过边界
    pub fn in_bounds(&self, point: Point) -> bool {
        point.x >= 0 && point.x < SCREEN_WIDTH
            && point.y >= 0 && point.y < SCREEN_HEIGHT
    }

    // 获取一个图块坐标的索引值，当坐标落在地图之外时返回一个错误提示
    pub fn try_idx(&self, point: Point) -> Option<usize> {
        if !self.in_bounds(point) {
            None
        } else {
            Some(map_idx(point.x, point.y))
        }
    }

    // 判断玩家是否可以进入一个图块
    pub fn can_enter_tile(&self, point: Point) -> bool {
        self.in_bounds(point)
            && self.tiles[map_idx(point.x, point.y)] == TileType::Floor
    }



    // 如果返回的是None，表示这个方向的走动行不通，如果返回的是Some，则包含目标图块的索引编号
    fn valid_exit(&self, loc: Point, delta: Point) -> Option<usize> {
        let destination = loc + delta;
        if self.in_bounds(destination) {
            if self.can_enter_tile(destination) {
                // 获取对应数组索引编号
                let idx = self.point2d_to_index(destination);
                Some(idx)
            } else {
                None
            }
        } else {
            None
        }
    }
}

// 地图映射
impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(SCREEN_WIDTH, SCREEN_HEIGHT)
    }

    fn in_bounds(&self, point: Point) -> bool {
        self.in_bounds(point)
    }
}

// 地图导航
impl BaseMap for Map {
    // SmallVec用于存储少量数据组成的列表，第1个位置表示存储的数据类型，第2个位置表示在退化成普通向量之前可以使用的内存大小。
    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        // 把地图中需要检测的图块的索引编号转化为x/y坐标对
        let location = self.index_to_point2d(idx);

        if let Some(idx) = self.valid_exit(location, Point::new(-1, 0)) {
            // 将作为出口的图块添加到出口列表中，代价为1.0，值越小，这条路径被选中的概率越大。
            exits.push((idx, 1.0))
        }
        if let Some(idx) = self.valid_exit(location, Point::new(1, 0)) {
            exits.push((idx, 1.0))
        }
        if let Some(idx) = self.valid_exit(location, Point::new(0, -1)) {
            exits.push((idx, 1.0))
        }
        if let Some(idx) = self.valid_exit(location, Point::new(0, 1)) {
            exits.push((idx, 1.0))
        }

        exits
    }

    // 计算直线距离
    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        DistanceAlg::Pythagoras
            .distance2d(
                self.index_to_point2d(idx1),
                self.index_to_point2d(idx2)
            )
    }

    fn is_opaque(&self, idx: usize) -> bool {
        // 定义墙是不透明的，地板是透明的
        self.tiles[idx] != TileType::Floor
    }
}

