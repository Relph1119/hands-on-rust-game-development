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
    ((y * SCREEN_HEIGHT) + x) as usize
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

    // 判断玩家是否可以进入一个图块
    pub fn can_enter_tile(&self, point: Point) -> bool {
        self.in_bounds(point)
            && self.tiles[map_idx(point.x, point.y)] == TileType::Floor
    }

    // 获取一个图块坐标的索引值，当坐标落在地图之外时返回一个错误提示
    pub fn try_idx(&self, point: Point) -> Option<usize> {
        if !self.in_bounds(point) {
            None
        } else {
            Some(map_idx(point.x, point.y))
        }
    }
}
