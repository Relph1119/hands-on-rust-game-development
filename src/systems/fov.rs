use crate::prelude::*;

#[system]
#[read_component(Point)]
#[write_component(FieldOfView)]
pub fn fov(
    ecs: &mut SubWorld,
    #[resource] map: &Map
){
    let mut views = <(&Point, &mut FieldOfView)>::query();
    /* 将脏数据进行更新，调用路径追踪算法
     * 路径追踪算法：在起点周围绘制一个想象出来的圆形，从起点出发，向圆形轮廓所经过的每一个图块引出一条直线，
     *    直线遇到的每一个图块都被认为是可见的，如果直线遇到一个不透明的图块，这条直线的绘制就结束了。
     */
    views.iter_mut(ecs)
        .filter(|(_, fov)| fov.is_dirty)
        .for_each(|(pos, fov)| {
            fov.visible_tiles = field_of_view_set(*pos, fov.radius, map);
            fov.is_dirty = false;
        });
}