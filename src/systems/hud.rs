use crate::prelude::*;

#[system]
#[read_component(Health)]
#[read_component(Player)]
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
    draw_batch.submit(10000).expect("Batch error");
}