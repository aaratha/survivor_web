use std::sync::Arc;

use bevy::input::mouse;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::ui::update;
use bevy::window::PrimaryWindow;
use bevy_prototype_lyon::prelude::*;
use rand::prelude::*;

// build commands:
// cargo build --release --target wasm32-unknown-unknown
// wasm-bindgen --out-dir ./webbuild/out/ --target web ./target/wasm32-unknown-unknown/release/web-game.wasm

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ShapePlugin))
        .insert_resource(Msaa::Sample4)
        .insert_resource(MousePosition {
            position: Vec2::ZERO,
        })
        .insert_resource(EnemySpawnTimer(Timer::from_seconds(
            2.0,
            TimerMode::Repeating,
        )))
        .insert_resource(FoodSpawnTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, setup)
        .add_systems(Update, (render))
        .add_systems(
            FixedUpdate,
            (
                handle_mouse,
                spawn_enemies,
                spawn_food,
                rope_collisions,
                enemy_collisions,
                update,
            ),
        )
        .run();
}

#[derive(Resource)]
struct MousePosition {
    position: Vec2,
}

#[derive(Component)]
struct Rope {
    points: Vec<Vec2>,
    prev_points: Vec<Vec2>,
    segment_length: f32,
    thickness: f32,
    elasticity: f32,
    color: Color,
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

        self.constrain_points();

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

#[derive(Component)]
struct Enemy {
    position: Vec2,
    position_prev: Vec2,
    radius: f32,
}

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

impl Enemy {
    fn new(position: Vec2) -> Self {
        Enemy {
            position,
            position_prev: position,
            radius: rand::thread_rng().gen_range(5.0..15.0),
        }
    }

    fn update(&mut self, target: Vec2, acceleration: f32) {
        let velocity = self.position - self.position_prev;
        let direction_to_target = (target - self.position).normalize();
        let acceleration = direction_to_target * acceleration; // Adjust the 0.1 value to change the acceleration strength
        let next_position = self.position + velocity / 1.008 + acceleration;
        self.position_prev = self.position;
        self.position = next_position;
    }
}

#[derive(Component)]
struct Food {
    position: Vec2,
    radius: f32,
}

#[derive(Resource)]
struct FoodSpawnTimer(Timer);

impl Food {
    fn new(position: Vec2) -> Self {
        Food {
            position,
            radius: 5.0,
        }
    }
}

#[derive(Component)]
struct Score {
    value: i32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(Rope::new(Vec2::ZERO, Vec2::new(50.0, 0.0), 5)); // Replace Vec2::ZERO with the appropriate start and end vectors, and 10 with the desired count
}

fn update(
    mut ropes: Query<&mut Rope>,
    mut enemies: Query<&mut Enemy>,
    mouse_pos: Res<MousePosition>,
) {
    let target = mouse_pos.position;
    for mut rope in ropes.iter_mut() {
        rope.update(mouse_pos.position);
        // Add any additional logic here if needed
    }
    for mut enemy in enemies.iter_mut() {
        enemy.update(target, 0.01);
    }
}

fn render(
    mut commands: Commands,
    ropes: Query<(Entity, &Rope)>,
    enemies: Query<(Entity, &Enemy)>,
    food: Query<(Entity, &Food)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut previous_points_query: Query<(Entity, &RopePoint)>,
    asset_server: Res<AssetServer>,
) {
    // Despawn previous rope points
    for (entity, _) in previous_points_query.iter_mut() {
        commands.entity(entity).despawn();
    }

    for (_, rope) in ropes.iter() {
        let points = rope.points.clone();

        let material_handle = materials.add(ColorMaterial::from(rope.color));

        for point in points.iter() {
            let mut radius = rope.thickness / 2.0;
            if *point == points[0] || *point == points[points.len() - 1] {
                radius = rope.thickness;
            }
            let circle = meshes.add(Mesh::from(Circle {
                radius: radius,
                ..default()
            }));
            commands
                .spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(circle.clone()),
                    material: material_handle.clone(),
                    transform: Transform::from_xyz(point.x, point.y, 0.),
                    ..default()
                })
                .insert(RopePoint);
        }
    }
    for (_, enemy) in enemies.iter() {
        let circle = meshes.add(Mesh::from(Circle {
            radius: enemy.radius,
            ..default()
        }));
        commands
            .spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(circle.clone()),
                material: materials.add(ColorMaterial::from(Color::RED)),
                transform: Transform::from_xyz(enemy.position.x, enemy.position.y, 0.),
                ..default()
            })
            .insert(RopePoint);
    }
    for (_, piece) in food.iter() {
        let circle = meshes.add(Mesh::from(Circle {
            radius: piece.radius,
            ..default()
        }));
        commands
            .spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(circle.clone()),
                material: materials.add(ColorMaterial::from(Color::BLUE)),
                transform: Transform::from_xyz(piece.position.x, piece.position.y, 0.),
                ..default()
            })
            .insert(RopePoint);
    }
    commands.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "Score: 0".to_string(),
                style: TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            }],
            ..Default::default()
        },
        ..Default::default()
    });
}

#[derive(Component)]
struct RopePoint;

fn handle_mouse(
    mut commands: Commands,
    mut ropes: Query<&mut Rope>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut mouse_pos: ResMut<MousePosition>,
) {
    if let Some(window) = q_windows.get_single().ok() {
        if let Some(cursor_position) = window.cursor_position() {
            mouse_pos.position = adjust_coords(q_windows, cursor_position);
        }
    }
}

fn adjust_coords(q_windows: Query<&Window, With<PrimaryWindow>>, mouse_pos: Vec2) -> Vec2 {
    if let Some(window) = q_windows.get_single().ok() {
        let window_size = Vec2::new(window.width(), window.height());
        let adjusted_position = mouse_pos - window_size / 2.0;
        let adjusted_position = Vec2::new(adjusted_position.x, -adjusted_position.y);
        return adjusted_position;
    }
    Vec2::ZERO
}

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<EnemySpawnTimer>,
    ropes: Query<&Rope>,
    mut enemies: Query<(Entity, &mut Enemy)>,
) {
    if let Ok(rope) = ropes.get_single() {
        // Get the position of rope point 0
        let rope_point_position = rope.points[0];
        timer.0.tick(time.delta());
        if timer.0.finished() {
            // Generate a random distance between 300 and 400 pixels
            let distance = rand::thread_rng().gen_range(100.0..=200.0);

            // Calculate the enemy position using polar coordinates
            let pos_x = rope_point_position.x + distance;
            let pos_y = rope_point_position.y + distance;

            // Spawn the enemy at the calculated position
            let enemy = commands.spawn(Enemy::new(Vec2::new(pos_x, pos_y))).id();
        }
        // Despawn enemies that go outside a radius of 400 pixels from rope point 0
        for (entity, mut enemy) in enemies.iter_mut() {
            let distance_to_rope_point_0 = enemy.position.distance(rope_point_position);
            if distance_to_rope_point_0 > 500.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn spawn_food(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<FoodSpawnTimer>,
    ropes: Query<&Rope>,
    mut food: Query<(Entity, &mut Food)>,
    mut score: Query<&Score>,
) {
    if let Ok(rope) = ropes.get_single() {
        // Get the position of rope point 0
        let rope_points = &rope.points;
        let thickness = rope.thickness;
        timer.0.tick(time.delta());
        if timer.0.finished() {
            // Generate a random distance between 100 and 300 pixels
            let distance = rand::thread_rng().gen_range(100.0..=300.0);

            // Calculate the food position using polar coordinates
            let pos_x = rope_points[0].x + distance;
            let pos_y = rope_points[0].y + distance;

            // Spawn the food at the calculated position
            let piece = commands.spawn(Food::new(Vec2::new(pos_x, pos_y)));
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
            }
        }
    }
}

fn rope_collisions(ropes: Query<&Rope>, mut enemies: Query<&mut Enemy>) {
    for rope in ropes.iter() {
        for point in &rope.points {
            for mut enemy in enemies.iter_mut() {
                let distance = point.distance(enemy.position);
                let collision_radius = rope.thickness / 2.0 + enemy.radius;
                if distance < collision_radius {
                    // Calculate the correction vector
                    let direction = (enemy.position - *point).normalize();
                    let correction = direction * (collision_radius - distance);

                    // Apply the correction to the enemy position
                    enemy.position += correction;
                }
            }
        }
    }
}

fn enemy_collisions(mut enemies: Query<&mut Enemy>) {
    let mut enemy_combinations = enemies.iter_combinations_mut();
    while let Some([mut enemy_a, mut enemy_b]) = enemy_combinations.fetch_next() {
        let distance = enemy_a.position.distance(enemy_b.position);
        let collision_radius = enemy_a.radius + enemy_b.radius; // Radius of the enemy collision sphere
        if distance < collision_radius {
            // Calculate the correction vector
            let direction = (enemy_a.position - enemy_b.position).normalize();
            let correction = direction * (collision_radius - distance) / 2.0;

            // Apply the correction to both enemy positions
            enemy_a.position += correction;
            enemy_b.position -= correction;
        }
    }
}
