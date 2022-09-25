use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    time::FixedTimestep,
};
use rand::Rng;

const TIME_STEP: f32 = 1.0 / 60.0;
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
const SNAKE_SIZE: Vec3 = Vec3::new(20.0, 20.0, 0.0);
const SNAKE_COLOR: Color = Color::rgb(0.1, 0.7, 0.1);
const SNAKE_SPEED: f32 = 700.0;
const INITIAL_SNAKE_DIRECTION: Vec2 = Vec2::new(-0.5, 0.0);
const FOOD_SIZE: Vec3 = Vec3::new(20.0, 20.0, 0.0);

// We set the z-value of the ball to 1 so it renders on top in the case of overlap
const FOOD_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
const FOOD_COLOR: Color = Color::rgb(0.1, 0.8, 0.1);

const LEFT_WALL: f32 = -450.0;
const RIGHT_WALL: f32 = 450.0;
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const WALL_THICKNESS: f32 = 10.0;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

#[derive(Component)]
struct Snake;

#[derive(Component)]
struct Collider;

#[derive(Default)]
struct CollisionEvent;

#[derive(Bundle)]
struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

#[derive(Component)]
struct Food;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

// Which side of the arena is this wall located on?
enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;

        // make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }

            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(check_for_collisions)
                .with_system(move_snake.before(check_for_collisions))
                .with_system(apply_velocity.before(check_for_collisions)),
        )
        .add_system(bevy::window::close_on_esc)
        .run();
}


fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // snake
    // let snake_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

    commands
        .spawn()
        .insert(Snake)
        .insert_bundle(SpriteBundle{
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: SNAKE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: SNAKE_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Velocity(INITIAL_SNAKE_DIRECTION.normalize() * SNAKE_SPEED)); 

    // Food
    let mut rng = rand::thread_rng();
    let food_x = rng.gen_range(-450.0..450.0);
    let food_y = rng.gen_range(-300.0..300.0);
    commands
        .spawn()
        .insert(Food)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                scale: FOOD_SIZE,
                translation: Vec3::new(food_x, food_y, 0.0),
                ..default()
            },
            sprite: Sprite {
                color: FOOD_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Collider);
        

    //walls
    commands.spawn_bundle(WallBundle::new(WallLocation::Left));
    commands.spawn_bundle(WallBundle::new(WallLocation::Right));
    commands.spawn_bundle(WallBundle::new(WallLocation::Bottom));
    commands.spawn_bundle(WallBundle::new(WallLocation::Top));

}

fn move_snake(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Snake>>,) {
        let (mut snake_velocity, mut snake_transform) = query.single_mut();
        let mut direction_x = 0.0;
        let mut direction_y = 0.0;

        if keyboard_input.pressed(KeyCode::Left) {
            direction_x -= 1.0;
            snake_velocity.x = -snake_velocity.x;
        }

        if keyboard_input.pressed(KeyCode::Right) {
            direction_x += 1.0;
            if(snake_velocity.x < 0.0) {
                snake_velocity.x = snake_velocity.x;
            }

        }
        if keyboard_input.pressed(KeyCode::Up) {
            direction_y += 1.0;
            snake_velocity.y = snake_velocity.y;

        }

        if keyboard_input.pressed(KeyCode::Down) {
            direction_y -= 1.0;
            snake_velocity.y = -snake_velocity.y;

        }

        // calculate the new horizontal paddle position based on plyaer input
        let new_snake_position = snake_transform.translation.x +  direction_x * SNAKE_SPEED * TIME_STEP;
        let new_snake_pos_vertical = snake_transform.translation.y + direction_y * SNAKE_SPEED * TIME_STEP;

        // Update the snake position,
        // make sure it does not cause the snake to leave the arena
        let left_bound = LEFT_WALL + WALL_THICKNESS + SNAKE_SIZE.x / 2.75;
        let right_bound = RIGHT_WALL - WALL_THICKNESS - SNAKE_SIZE.x / 2.75;
        let top_bound = TOP_WALL - WALL_THICKNESS - SNAKE_SIZE.y / 2.75;
        let bottom_bound = BOTTOM_WALL + WALL_THICKNESS + SNAKE_SIZE.y / 2.75;

        snake_transform.translation.x = new_snake_position.clamp(left_bound, right_bound);
        snake_transform.translation.y = new_snake_pos_vertical.clamp(bottom_bound, top_bound);
    }

impl WallBundle {
    // This "builder method" allows us to reuse logic across out wall entities,
    // making our code easier to read and less prone to bugs when we change the logic

    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.__rust_force_expr!
                    // see https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
        }
      
}

fn check_for_collisions(
    mut commands: Commands,
    mut snake_query: Query<(&mut Velocity, &Transform), With<Snake>>,
    collider_query: Query<(Entity, &Transform, Option<&Food>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut snake_velocity, snake_transform) = snake_query.single_mut();
    let snake_size = snake_transform.scale.truncate();

    // check collision with walls
    for (collider_entity, transform, maybe_food) in &collider_query {
        let collision = collide(
            snake_transform.translation,
            snake_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            // Sends a collision event so that other systems can react to the collision
            collision_events.send_default();

            // Food should be despawned and increment the scoreboard on collision
            if maybe_food.is_some() {
                // scoreboard.score += 1;
                commands.entity(collider_entity).despawn();
                let mut rng = rand::thread_rng();
                let food_x = rng.gen_range(-450.0..450.0);
                let food_y = rng.gen_range(-300.0..300.0);
                commands.spawn().insert(Food).insert_bundle(SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(food_x, food_y, 0.0),
                        scale: FOOD_SIZE,
                        ..default()
                    },
                    sprite: Sprite {
                        color: FOOD_COLOR,
                        ..default()
                    },
                    ..default()
                })
                .insert(Collider);
                // increase snake's tail



            }
        }
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}