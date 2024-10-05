pub use crate::prelude::*;

// 渲染实体
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Render {
    // 同时存储前景色和背景色
    pub color: ColorPair,
    // 标签，指示包含这个组件的实体是玩家角色对应的实体
    pub glyph: FontCharType
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToMove {
    pub entity: Entity,
    pub destination: Point
}