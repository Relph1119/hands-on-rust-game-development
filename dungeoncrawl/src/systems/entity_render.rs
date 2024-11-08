use crate::prelude::*;

/*
 * Camera用于计算实体在屏幕坐标系下坐标的偏移量
 * Point用于获取实体的位置
 * Render用于描述实体的展现形式
 */
#[system]
#[read_component(Point)]
#[read_component(Render)]
#[read_component(FieldOfView)]
#[read_component(Player)]
pub fn entity_render(#[resource] camera: &Camera,
                     ecs: &SubWorld,) {
    let mut renderables = <(&Point, &Render)>::query();
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    // 开启一个新批量绘制
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(1);
    let offset = Point::new(camera.left_x, camera.top_y);
    // 获取玩家的视野
    let player_fov = fov.iter(ecs).nth(0).unwrap();

    renderables
        .iter(ecs)
        .filter(|(pos, _)| player_fov.visible_tiles.contains(&pos))
        .for_each(|(pos, render)| {
            draw_batch.set(
                *pos - offset,
                render.color,
                render.glyph
            );
        });
    // 地图可能包含4000个以上元素，使用5000作为排序序号
    draw_batch.submit(5000).expect("Batch error");
}