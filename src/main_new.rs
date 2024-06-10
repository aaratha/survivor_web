// Import necessary modules
use bevy::math::Vec3; // Add this import statement
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_polyline::prelude::*;
use rand::prelude::*;

// Constants
const SUBSTEPS: i32 = 5;

// Main function
fn main() {
    // Create the Bevy app
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa::Sample4)
        .insert_resource(EnemySpawnTimer(Timer::from_seconds(
            2.0,
            TimerMode::Repeating,
        )))
        .insert_resource(FoodSpawnTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_mouse, update_enemies, update_food, render))
        .run();
}

// Components and resources
#[derive(Resource)]
struct MousePosition {
    position: Vec2,
}

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

#[derive(Resource)]
struct FoodSpawnTimer(Timer);

// Setup function
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(Rope::new(Vec2::ZERO, Vec2::new(50.0, 0.0), 5)); // Adjust coordinates and count
}

// Systems
fn handle_mouse(
    mut commands: Commands,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut mouse_pos: ResMut<MousePosition>,
) {
    if let Some(window) = q_windows.get_single().ok() {
        if let Some(cursor_position) = window.cursor_position() {
            mouse_pos.position = adjust_coords(q_windows, cursor_position);
        }
    }
}

fn update_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<EnemySpawnTimer>,
    ropes: Query<&Rope>,
    mut enemies: Query<(Entity, &mut Enemy)>,
) {
    if let Ok(rope) = ropes.get_single() {
        let rope_point_position = rope.points[0];
        timer.0.tick(time.delta());
        if timer.0.finished() {
            let distance = rand::thread_rng().gen_range(100.0..=200.0);
            let pos_x = rope_point_position.x + distance;
            let pos_y = rope_point_position.y + distance;
            let enemy = commands.spawn(Enemy::new(Vec2::new(pos_x, pos_y))).id();
            println!("Enemy spawned");
        }
        for (entity, mut enemy) in enemies.iter_mut() {
            let distance_to_rope_point_0 = enemy.position.distance(rope_point_position);
            if distance_to_rope_point_0 > 500.0 {
                commands.entity(entity).despawn();
                println!("Enemy despawned");
            }
        }
    }
}

fn update_food(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<FoodSpawnTimer>,
    ropes: Query<&Rope>,
    mut food: Query<(Entity, &mut Food)>,
) {
    if let Ok(rope) = ropes.get_single() {
        let rope_points = &rope.points;
        let thickness = rope.thickness;
        timer.0.tick(time.delta());
        if timer.0.finished() {
            let distance = rand::thread_rng().gen_range(100.0..=300.0);
            let pos_x = rope_points[0].x + distance;
            let pos_y = rope_points[0].y + distance;
            let piece = commands.spawn(Food::new(Vec2::new(pos_x, pos_y)));
            println!("Food spawned");
        }
        for (entity, mut piece) in food.iter_mut() {
            let mut too_close = false;
            for point in rope_points.iter() {
                let distance_to_rope_point = piece.position.distance(*point);
                if distance_to_rope_point < thickness + piece.radius {
                    too_close = true;
                    break;
                }
            }
            if too_close {
                commands.entity(entity).despawn();
                println!("Food despawned");
            }
        }
    }
}

fn render(
    mut commands: Commands,
    ropes: Query<(Entity, &Rope)>,
    enemies: Query<(Entity, &Enemy)>,
    food: Query<(Entity, &Food)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, rope) in ropes.iter() {
        let rope_points_3d: Vec<Vec3> = rope
            .points
            .iter()
            .map(|point| Vec3::new(point.x, point.y, 0.0))
            .collect();
        let rope_mesh = meshes.add(Mesh::from(Polyline {
            vertices: rope_points_3d,
            closed: false,
        }));
        let material = materials.add(Color::WHITE.into());
        commands.entity(entity).insert_bundle(PbrBundle {
            mesh: rope_mesh,
            material: material.clone(),
            ..Default::default()
        });
    }
    for (entity, enemy) in enemies.iter() {
        let enemy_mesh = meshes.add(Mesh::from(shape::Circle {
            radius: enemy.radius,
            ..Default::default()
        }));
        let material = materials.add(Color::RED.into());
        commands.entity(entity).insert_bundle(PbrBundle {
            mesh: enemy_mesh,
            material: material.clone(),
            transform: Transform::from_translation(Vec3::new(
                enemy.position.x,
                enemy.position.y,
                0.0,
            )),
            ..Default::default()
        });
    }
    for (entity, piece) in food.iter() {
        // Render food
        let food_mesh = meshes.add(Mesh::from(shape::Circle {
            radius: piece.radius,
            ..Default::default()
        }));
        let material = materials.add(Color::GREEN.into());
        commands.entity(entity).insert_bundle(PbrBundle {
            mesh: food_mesh,
            material: material.clone(),
            transform: Transform::from_translation(Vec3::new(
                piece.position.x,
                piece.position.y,
                0.0,
            )),
            ..Default::default()
        });
    }
}

// Helper functions
fn adjust_coords(q_windows: Query<&Window, With<PrimaryWindow>>, mouse_pos: Vec2) -> Vec2 {
    if let Some(window) = q_windows.get_single().ok() {
        let window_size = Vec2::new(window.width(), window.height());
        let adjusted_position = mouse_pos - window_size / 2.0;
        let adjusted_position = Vec2::new(adjusted_position.x, -adjusted_position.y);
        return adjusted_position;
    }
    Vec2::ZERO
}

#[derive(Debug, Component, Clone, PartialEq)]
struct Rope {
    points: Vec<Vec2>,
    prev_points: Vec<Vec2>,
    segment_length: f32,
    thickness: f32,
    elasticity: f32,
    color: Color,
}

#[derive(Component)]
struct Enemy {
    position: Vec2,
    position_prev: Vec2,
    radius: f32,
}

#[derive(Component)]
struct Food {
    position: Vec2,
    radius: f32,
}

impl Rope {
    fn new(start: Vec2, end: Vec2, count: usize) -> Self {
        let length = start.distance(end);
        let segment_length = length / (count as f32 - 1.0);
        let direction = (end - start).normalize();

        let points: Vec<Vec2> = (0..count)
            .map(|i| start + direction * segment_length * i as f32)
            .collect();

        let prev_points = points.clone();

        Rope {
            points,
            prev_points,
            segment_length,
            thickness: 6.0,
            elasticity: 5.0,
            color: Color::WHITE,
        }
    }

    fn update(&mut self, mouse_pos: Vec2) {
        self.update_rope(mouse_pos);
    }

    fn update_rope(&mut self, mouse_pos: Vec2) {
        for i in 1..self.points.len() {
            let current = self.points[i];
            let prev = self.prev_points[i];
            let velocity = current - prev;
            let next_position = current + velocity / 1.008; // Apply gravity here if needed
            self.prev_points[i] = self.points[i];
            self.points[i] = next_position;
        }

        for _ in 0..SUBSTEPS {
            self.constrain_points();
        }

        self.points[0] = mouse_pos;
    }

    fn constrain_points(&mut self) {
        let count = self.points.len();
        for _ in 0..3 {
            for i in 0..(count - 1) {
                let point_a = self.points[i];
                let point_b = self.points[i + 1];
                let delta = point_b - point_a;
                let distance = delta.length();
                let difference = self.segment_length - distance;
                let correction = delta.normalize() * (difference / self.elasticity);
                if i != 0 {
                    self.points[i] -= correction;
                }
                self.points[i + 1] += correction;
            }
        }
    }
}

impl Enemy {
    fn new(position: Vec2) -> Self {
        Enemy {
            position,
            position_prev: position,
            radius: rand::thread_rng().gen_range(5.0..15.0),
        }
    }

    fn update(&mut self, target: Vec2, acceleration: f32) {
        for _ in 0..SUBSTEPS {
            let velocity = self.position - self.position_prev;
            let direction_to_target = (target - self.position).normalize();
            let acceleration = direction_to_target * acceleration; // Adjust the 0.1 value to change the acceleration strength
            let next_position = self.position + velocity / 1.008 + acceleration;
            self.position_prev = self.position;
            self.position = next_position;
        }
    }
}

impl Food {
    fn new(position: Vec2) -> Self {
        Food {
            position,
            radius: 5.0,
        }
    }
}
