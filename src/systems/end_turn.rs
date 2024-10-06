use crate::prelude::*;

#[system]
#[read_component(Health)]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(AmuletOfYala)]
pub fn end_turn(ecs: &SubWorld,
                #[resource] turn_state: &mut TurnState,
                #[resource] map: &Map) {
    let mut player_hp = <(&Health, &Point)>::query().filter(component::<Player>());
    // 获得护身符的位置
    let mut amulet = <&Point>::query().filter(component::<AmuletOfYala>());
    // let amulet_pos =  amulet.iter(ecs).nth(0).unwrap();
    let amulet_default = Point::new(-1, -1);
    let amulet_pos = amulet.iter(ecs).nth(0).unwrap_or(&amulet_default);

    let current_state = turn_state.clone();
    // 状态转移
    let mut new_state = match turn_state {
        TurnState::AwaitingInput => return,
        TurnState::PlayerTurn => TurnState::MonsterTurn,
        TurnState::MonsterTurn => TurnState::AwaitingInput,
        _ => current_state
    };
    player_hp.iter(ecs).for_each(|(hp, pos)| {
        // 如果玩家角色的生命值已经耗尽，游戏结束
        if hp.current < 1 {
            new_state = TurnState::GameOver;
        }
        if pos == amulet_pos {
            new_state = TurnState::Victory;
        }
        // 走进地下城的下一层
        let idx = map.point2d_to_index(*pos);
        if map.tiles[idx] == TileType::Exit {
            new_state = TurnState::NextLevel;
        }
    });
    *turn_state = new_state;
}