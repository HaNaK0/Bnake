use bevy::{core::FixedTimestep, prelude::*};
use rand::prelude::random;
use wasm_bindgen::prelude::*;

const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
        }
    }
}

struct SnakeHead {
    direction: Direction,
}

#[derive(Debug, SystemLabel, Hash, PartialEq, Eq, Clone)]
pub enum SnakeMovement {
    Input,
    Movement,
    Eating,
    Growth,
}

struct Materials {
    head_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
}

struct SnakeSegment;

#[derive(Default)]
struct SnakeSegments(Vec<Entity>);

struct Food;

struct GrowthEvent;

struct GameOverEvent;

#[derive(Default)]
struct LastTailPosition(Option<Position>);

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();
    app.insert_resource(WindowDescriptor {
            title: "Bnake!".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup", SystemStage::single(spawn_snake.system()))
        .add_system(
            snake_movement_input
                .system()
                .label(SnakeMovement::Input)
                .before(SnakeMovement::Movement),
        )
        .add_system(game_over.system().after(SnakeMovement::Movement))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.150))
                .with_system(snake_movement.system().label(SnakeMovement::Movement))
                .with_system(
                    snake_eating
                        .system()
                        .label(SnakeMovement::Eating)
                        .after(SnakeMovement::Movement)
                )
                .with_system(
                    snake_growth
                        .system()
                        .label(SnakeMovement::Growth)
                        .after(SnakeMovement::Eating)
                ),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(size_scaling.system())
                .with_system(position_translation.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(food_spawner.system()),
        )
        .add_plugins(DefaultPlugins)
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>();
        
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    
    app.run()
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    info!("Hello World!");
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
        segment_material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
        food_material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
    })
}

fn spawn_snake(
    mut commands: Commands, 
    materials: Res<Materials>,
    mut segments: ResMut<SnakeSegments>,
) {
    segments.0 = vec![
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.head_material.clone(),
                sprite: Sprite::new(Vec2::new(10.0, 10.0)),
                ..Default::default()
            })
            .insert(
                SnakeHead {
                    direction: Direction::Up,
                }
            )
            .insert(SnakeSegment)
            .insert(Position {x: 3, y: 3})
            .insert(Size::square(0.8))
            .id(),
        spawn_segment(
            commands, 
            &materials.segment_material, 
            Position {x: 3, y: 2},
        )
    ];
}

fn snake_movement(
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        
        let mut head_position = positions.get_mut(head_entity).unwrap();
        match &head.direction {
            Direction::Left => {
                head_position.x -= 1
            }
            Direction::Up => {
                head_position.y += 1
            }
            Direction::Right => {
                head_position.x += 1
            }
            Direction::Down => {
                head_position.y -= 1
            }
        }

        if head_position.x < 0
            || head_position.y < 0
            || head_position.x as u32 >= ARENA_WIDTH
            || head_position.y as u32 >= ARENA_HEIGHT 
        {
            game_over_writer.send(GameOverEvent)
        }

        if segment_positions.contains(&head_position) {
            game_over_writer.send(GameOverEvent);
        }

        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos, segment)|{
                *positions.get_mut(*segment).unwrap() = *pos;
            });
        
        last_tail_position.0 = Some(*segment_positions.last().unwrap());
    }


}

fn snake_movement_input(keyboard_input: Res<Input<KeyCode>>, mut heads: Query<&mut SnakeHead>) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else {
            head.direction
        };

        if dir != head.direction.opposite() {
            head.direction = dir
        }
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width(),
            sprite_size.height / ARENA_HEIGHT as f32 * window.height(),
        )
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn food_spawner(mut commands: Commands, materials: Res<Materials>) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.food_material.clone(),
            ..Default::default()
        })
        .insert(Food)
        .insert(Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_WIDTH as f32) as i32,
        })
        .insert(Size::square(0.8));
}

fn spawn_segment(
    mut commands: Commands,
    material: &Handle<ColorMaterial>,
    position: Position,
) -> Entity {
    commands
        .spawn_bundle( SpriteBundle {
            material: material.clone(),
            ..Default::default()
        })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
}

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>
) {
    for head_position in head_positions.iter() {
        for (entity, food_pos) in food_positions.iter() {
            if head_position == food_pos {
                commands.entity(entity).despawn();
                growth_writer.send(GrowthEvent);
            }
        }
    }
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
    materials: Res<Materials>,
) {
    if growth_reader.iter().next().is_some() {
        segments.0.push(spawn_segment(
            commands, 
            &materials.segment_material, 
            last_tail_position.0.unwrap()
        ))
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    materials: Res<Materials>,
    segment_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegments>>
){
    if reader.iter().next().is_some() {
        for entity in food.iter().chain(segments.iter()) {
            commands.entity(entity).despawn()
        }
        spawn_snake(commands, materials, segment_res)
    }
}
