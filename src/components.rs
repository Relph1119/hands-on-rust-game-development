use std::collections::HashSet;
pub use crate::prelude::*;

// 渲染实体
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Render {
    // 同时存储前景色和背景色
    pub color: ColorPair,
    // 标签，指示包含这个组件的实体是玩家角色对应的实体
    pub glyph: FontCharType,
}

// 玩家角色实体
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Player;

// 怪物实体
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Enemy;

// 随机移动实体
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovingRandomly;

// 标记怪物正在追逐玩家角色
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChasingPlayer;

// 移动意图实体
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToMove {
    pub entity: Entity,
    pub destination: Point,
}

// 攻击意图实体
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToAttack {
    // 攻击者
    pub attacker: Entity,
    // 受害者
    pub victim: Entity,
}

// 生命值实体
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Health {
    // 当前生命值
    pub current: i32,
    // 最大生命值
    pub max: i32,
}

// 悬浮提示
#[derive(Clone, PartialEq)]
pub struct Name(pub String);

// 物品
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Item;

// 用来赢得游戏的物品：亚拉的护身符
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmuletOfYala;

#[derive(Clone, Debug, PartialEq)]
pub struct FieldOfView {
    // 实体在地图上可见的图块
    pub visible_tiles: HashSet<Point>,
    // 在每个方向上课可以看到的图块数量
    pub radius: i32,
    // 在数据需要更新的时候，将其标记为“脏数据”
    pub is_dirty: bool,
}

impl FieldOfView {
    pub fn new(radius: i32) -> Self {
        Self {
            visible_tiles: HashSet::new(),
            radius,
            is_dirty: true
        }
    }

    pub fn clone_dirty(&self) -> Self {
        Self {
            visible_tiles: HashSet::new(),
            radius: self.radius,
            is_dirty: true
        }
    }
}

// 治疗药水
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProvidesHealing {
    // 药水可以恢复的生命值
    pub amount: i32
}

// 地下城地图
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProvidesDungeonMap;

// 捡起物品
#[derive(Clone, PartialEq)]
pub struct Carried(pub Entity);

// 使用物品
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActivateItem {
    pub used_by: Entity,
    pub item: Entity
}