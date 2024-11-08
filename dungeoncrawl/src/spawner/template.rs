use std::collections::HashSet;
use std::fs::File;
use ron::de::from_reader;
use serde::Deserialize;
use crate::prelude::*;

#[derive(Clone, Deserialize, Debug)]
pub struct Template {
    // 实体类型
    pub entity_type: EntityType,
    // 实体可以在哪些关卡中出现的关卡编号
    pub levels: HashSet<usize>,
    // 物品出现的概率
    pub frequency: i32,
    // 实体名字
    pub name: String,
    // 渲染实体的字符
    pub glyph: char,
    // 该物品提供的特殊效果
    pub provides: Option<Vec<(String, i32)>>,
    // 怪物的生命值
    pub hp: Option<i32>,
    // 基础伤害
    pub base_damage: Option<i32>
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub enum EntityType {
    // 怪物
    Enemy,
    // 物品
    Item,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Templates {
    // 存储所有Template的向量
    pub entities: Vec<Template>,
}

impl Templates {
    // 加载配置文件
    pub fn load() -> Self {
        let file = File::open("resources/template.ron").expect("Failed opening file");
        from_reader(file).expect("Unable to load templates")
    }

    pub fn spawn_entities(&self,
                          ecs: &mut World,
                          rng: &mut RandomNumberGenerator,
                          level: usize, spawn_points: &[Point]) {
        // 存储在当前关卡中生成的实体
        let mut available_entities = Vec::new();
        // 获得可用实体的列表，包括治疗药水、地图、妖精、兽人等实体，并按照各个实体的频率数保存在列表中。
        self.entities
            .iter().filter(|e| e.levels.contains(&level))
            .for_each(|t| {
                for _ in 0..t.frequency {
                    available_entities.push(t);
                }
            });

        // 将实体放置在坐标点上
        let mut commands = CommandBuffer::new(ecs);
        spawn_points.iter().for_each(|pt| {
            if let Some(entity) = rng.random_slice_entry(&available_entities) {
                self.spawn_entity(pt, entity, &mut commands);
            }
        });
        commands.flush(ecs);
    }

    fn spawn_entity(&self,
                    pt: &Point,
                    template: &Template,
                    commands: &mut CommandBuffer) {
        // 存储与渲染相关的信息，包括位置、渲染信息、名字
        let entity = commands.push((
            pt.clone(),
            Render {
                color: ColorPair::new(WHITE, BLACK),
                glyph: to_cp437(template.glyph),
            },
            Name(template.name.clone())));
        // 添加标签，使用命令缓冲区添加组件
        match template.entity_type {
            EntityType::Item => commands.add_component(entity, Item {}),
            EntityType::Enemy => {
                commands.add_component(entity, Enemy {});
                commands.add_component(entity, FieldOfView::new(6));
                commands.add_component(entity, ChasingPlayer {});
                commands.add_component(entity, Health {
                    current: template.hp.unwrap(),
                    max: template.hp.unwrap(),
                });
            }
        }
        // 添加特殊效果对应的组件，包括治疗药水、地图
        if let Some(effects) = &template.provides {
            effects.iter().for_each(|(provides, n)| {
                match provides.as_str() {
                    "Healing" => commands.add_component(entity, ProvidesHealing { amount: *n }),
                    "MagicMap" => commands.add_component(entity, ProvidesDungeonMap {}),
                    _ => println!("Warning: we don't know how to provide {}", provides)
                }
            });
        }

        // 添加伤害值信息
        if let Some(damage) = &template.base_damage {
            commands.add_component(entity, Damage(*damage));

            if template.entity_type == EntityType::Item {
                commands.add_component(entity, Weapon{});
            }
        }
    }
}
