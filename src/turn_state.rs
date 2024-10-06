#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnState {
    // 等待输入
    AwaitingInput,
    // 玩家移动
    PlayerTurn,
    // 怪物移动
    MonsterTurn,
    // 游戏结束
    GameOver,
    // 游戏胜利
    Victory
}