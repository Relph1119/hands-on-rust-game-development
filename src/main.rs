mod map;
mod map_builder;
mod camera;
mod components;
mod spawner;
mod systems;
mod turn_state;

mod prelude {
    pub use bracket_lib::prelude::*;
    pub use legion::*;
    pub use legion::world::SubWorld;
    pub use legion::systems::CommandBuffer;

    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;

    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;

    pub use crate::map::*;

    pub use crate::map_builder::*;

    pub use crate::camera::*;

    pub use crate::components::*;

    pub use crate::spawner::*;

    pub use crate::systems::*;

    pub use crate::turn_state::*;
}

use prelude::*;

struct State {
    // 存储所有的实体和组件，Entity Component System实体组件系统
    ecs: World,
    resources: Resources,
    input_systems: Schedule,
    player_systems: Schedule,
    monster_systems: Schedule
}

impl State {
    fn new() -> Self {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let map_builder = MapBuilder::new(&mut rng);
        // 设置玩家角色
        spawn_player(&mut ecs, map_builder.player_start);
        // 设置护身符
        spawn_amulet_of_yala(&mut ecs, map_builder.amulet_start);
        // 将怪物放置在地图上
        map_builder.monster_spawns.iter().for_each(|pos| spawn_monster(&mut ecs, &mut rng, *pos));
        resources.insert(map_builder.map);
        resources.insert(Camera::new(map_builder.player_start));
        resources.insert(TurnState::AwaitingInput);
        resources.insert(map_builder.theme);
        Self {
            ecs,
            resources,
            input_systems: build_input_scheduler(),
            player_systems: build_player_scheduler(),
            monster_systems: build_monster_scheduler()
        }
    }

    fn reset_game_state(&mut self) {
        self.ecs = World::default();
        self.resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let map_builder = MapBuilder::new(&mut rng);
        spawn_player(&mut self.ecs, map_builder.player_start);
        spawn_amulet_of_yala(&mut self.ecs, map_builder.amulet_start);
        map_builder.monster_spawns.iter().for_each(|pos| spawn_monster(&mut self.ecs, &mut rng, *pos));
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
    }

    fn game_over(&mut self, ctx: &mut BTerm) {
        // 展示在平视显示区
        ctx.set_active_console(2);
        ctx.print_color_centered(2, RED, BLACK, "Your quest has ended.");
        ctx.print_color_centered(4, WHITE, BLACK,
                                 "Slain by a monster, your hero's journey has come to a premature end.");
        ctx.print_color_centered(5, WHITE, BLACK,
                                 "The Amulet of Yala remains unclaimed, and your home town is not saved.");
        ctx.print_color_centered(8, YELLOW, BLACK,
                                 "Don't worry, you can always try again with a new hero.");
        ctx.print_color_centered(9, GREEN, BLACK,
                                 "Press 1 to play again.");
        // 使用1号键，避免不小心跳过游戏结束画面
        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.reset_game_state();
        }
    }

    fn victory(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(2);
        ctx.print_color_centered(2, GREEN, BLACK, "You have won!");
        ctx.print_color_centered(4, WHITE, BLACK,
                                 "You put on the Amulet of Yala and feel its power course through \
            your veins.");
        ctx.print_color_centered(5, WHITE, BLACK,
                                 "Your town is saved, and you can return to your normal life.");
        ctx.print_color_centered(7, GREEN, BLACK, "Press 1 to \
            play again.");
        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.reset_game_state();
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        // 清空每一个图层
        // 0：地图图层
        ctx.set_active_console(0);
        ctx.cls();
        // 1：实体图层
        ctx.set_active_console(1);
        ctx.cls();
        // 2：平视显示区图层
        ctx.set_active_console(2);
        ctx.cls();
        // 将键盘的输入状态作为一个资源加入到资源列表中
        self.resources.insert(ctx.key);
        // 从地图图层中获得鼠标位置
        ctx.set_active_console(0);
        self.resources.insert(Point::from_tuple(ctx.mouse_pos()));

        // 执行各个系统的执行计划
        let current_state = self.resources.get::<TurnState>().unwrap().clone();
        match current_state {
            TurnState::AwaitingInput => self.input_systems.execute(&mut self.ecs, &mut self.resources),
            TurnState::PlayerTurn => self.player_systems.execute(&mut self.ecs, &mut self.resources),
            TurnState::MonsterTurn => self.monster_systems.execute(&mut self.ecs, &mut self.resources),
            TurnState::GameOver => self.game_over(ctx),
            TurnState::Victory => self.victory(ctx)
        }
        // 批量渲染
        render_draw_buffer(ctx).expect("Render error");
    }
}

fn main() -> BError {
    /* with_dimensions：添加控制台尺寸
     * with_tile_dimensions：设置图块的尺寸
     * with_resource_path：设置资源存放目录
     * with_font：设置加载的字体文件和尺寸
     * with_simple_console：添加一个新图层，用于绘制地图
     * with_simple_console_no_bg：添加一个透明图层，用于绘制玩家角色
     */
    let context = BTermBuilder::new()
        .with_title("Dungeon Crawler")
        .with_fps_cap(30.0)
        .with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .with_tile_dimensions(32, 32)
        .with_resource_path("resources/")
        .with_font("dungeonfont.png", 32, 32)
        .with_font("terminal8x8.png", 8, 8)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png") //地图
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png") // 实体
        .with_simple_console_no_bg(SCREEN_WIDTH*2, SCREEN_HEIGHT*2, "terminal8x8.png") // 平视显示区
        .build()?;
    main_loop(context, State::new())
}
