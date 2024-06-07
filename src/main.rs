use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::ui::update;
use bevy::window::PrimaryWindow;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (render, update_rope_system, handle_mouse))
        .run();
}

#[derive(Component)]
enum Direction {
    Up,
    Down,
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
            thickness: 4.0,
            elasticity: 1.1,
            color: Color::WHITE,
        }
    }

    fn update(&mut self, substeps: i32) {
        self.update_rope(substeps);
    }

    fn update_rope(&mut self, substeps: i32) {
        for i in 1..self.points.len() {
            let current = self.points[i];
            let prev = self.prev_points[i];
            let velocity = current - prev;
            let next_position = current + velocity / 1.008; // Apply gravity here if needed
            self.prev_points[i] = self.points[i];
            self.points[i] = next_position;
        }

        for _ in 0..substeps {
            self.constrain_points();
        }
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
                let correction = delta.normalize() * (difference / 15.0);
                if i != 0 {
                    self.points[i] -= correction;
                }
                self.points[i + 1] += correction;
            }
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("branding/icon.png"),
            transform: Transform::from_xyz(100., 0., 0.),
            ..default()
        },
        Direction::Up,
    ));
    commands.spawn(Rope::new(Vec2::ZERO, Vec2::new(100.0, 100.0), 10)); // Replace Vec2::ZERO with the appropriate start and end vectors, and 10 with the desired count
}

fn update_rope_system(mut ropes: Query<&mut Rope>) {
    let substeps = 5; // Number of substeps for more accurate updates
    for mut rope in ropes.iter_mut() {
        rope.update(substeps);
        // Add any additional logic here if needed
    }
}

fn render(
    mut commands: Commands,
    ropes: Query<(Entity, &Rope)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut previous_points_query: Query<(Entity, &RopePoint)>,
) {
    // Despawn previous rope points
    for (entity, _) in previous_points_query.iter_mut() {
        commands.entity(entity).despawn();
    }

    for (_, rope) in ropes.iter() {
        let points = rope.points.clone();
        let circle = meshes.add(Mesh::from(shape::Circle {
            radius: rope.thickness / 2.0,
            ..default()
        }));
        let material_handle = materials.add(ColorMaterial::from(rope.color));

        for point in points {
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
}

#[derive(Component)]
struct RopePoint;

fn handle_mouse(
    mut commands: Commands,
    mut ropes: Query<&mut Rope>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Some(window) = q_windows.get_single().ok() {
        if let Some(cursor_position) = window.cursor_position() {
            let window_size = Vec2::new(window.width(), window.height());
            let adjusted_position = cursor_position - window_size / 2.0;
            let adjusted_position = Vec2::new(adjusted_position.x, -adjusted_position.y);

            for mut rope in ropes.iter_mut() {
                let start = adjusted_position;
                rope.points[0] = start;
            }
        }
    }
}
