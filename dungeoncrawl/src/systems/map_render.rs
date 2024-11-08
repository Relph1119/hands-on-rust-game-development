use crate::prelude::*;

#[system]
#[read_component(FieldOfView)]
#[read_component(Player)]
pub fn map_render(#[resource] map: &Map,
                  #[resource] camera: &Camera,
                  #[resource] theme: &Box<dyn MapTheme>,
                  ecs: &SubWorld) {
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    // 开启一个新的批量绘制
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(0);
    // 获取玩家的视野
    let player_fov = fov.iter(ecs).nth(0).unwrap();

    // 地图渲染
    for y in camera.top_y..camera.bottom_y {
        for x in camera.left_x..camera.right_x {
            let pt = Point::new(x, y);
            let offset = Point::new(camera.left_x, camera.top_y);
            let idx = map_idx(x, y);
            // 检查图块是否在玩家视野中，或者是否在已揭示的图块列表中
            if map.in_bounds(pt) && player_fov.visible_tiles.contains(&pt) | map.revealed_tiles[idx] {
                let tint = if player_fov.visible_tiles.contains(&pt) {
                    WHITE
                } else {
                    DARK_GRAY
                };
                // 获取主题风格并渲染
                let glyph = theme.tile_to_render(map.tiles[idx]);
                draw_batch.set(
                    pt - offset,
                    ColorPair::new(tint, BLACK),
                    glyph
                );
            }
        }
    }
    // 提交一个批次，最先被渲染。
    draw_batch.submit(0).expect("Batch error");
}