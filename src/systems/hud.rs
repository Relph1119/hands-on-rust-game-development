use crate::prelude::*;

#[system]
#[read_component(Health)]
#[read_component(Player)]
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(Name)]
pub fn hud(ecs: &SubWorld) {
    // 筛选出玩家角色对应的组件
    let mut health_query = <&Health>::query().filter(component::<Player>());
    let player_health = health_query.iter(ecs).nth(0).unwrap();

    let mut draw_batch = DrawBatch::new();
    // 批量绘制平视显示区
    draw_batch.target(2);
    draw_batch.print_centered(1, "Explore the Dungeon. Cursor keys to move.");
    // 血条起始坐标为0、血条宽度、血条当前值、血条最大值、显示血条（空为红色、满为黑色）
    draw_batch.bar_horizontal(
        Point::zero(),
        SCREEN_WIDTH*2,
        player_health.current,
        player_health.max,
        ColorPair::new(RED, BLACK)
    );
    draw_batch.print_color_centered(
        0,
        format!(" Health: {} / {}", player_health.current, player_health.max),
        ColorPair::new(WHITE, RED)
    );

    let player = <(Entity, &Player)>::query().iter(ecs)
        .find_map(|(entity, _player)| Some(*entity)).unwrap();
    let mut item_query = <(&Item, &Name, &Carried)>::query();
    // 物品列表的渲染位置，在屏幕的第3行
    let mut y = 3;
    item_query.iter(ecs).filter(|(_, _, carried)| carried.0 == player)
        .for_each(|(_, name, _)| {
            // 显示在第3列第y行，显示获取物品的列表
            draw_batch.print(Point::new(3, y), format!("{} : {}", y - 2, &name.0));
            y += 1;
        });
    if y > 3 {
        // 添加物品列表标题
        draw_batch.print_color(Point::new(3, 2), "Items carried", ColorPair::new(YELLOW, BLACK));
    }

    // 显示当前关卡
    let (_player, map_level) = <(Entity, &Player)>::query()
        .iter(ecs)
        .find_map(|(entity, player)| Some((*entity, player.map_level))).unwrap();
    draw_batch.print_color_right(
        Point::new(SCREEN_WIDTH*2, 1),
        format!("Dungeon Level: {}", map_level + 1),
        ColorPair::new(YELLOW, BLACK));

    draw_batch.submit(10000).expect("Batch error");
}