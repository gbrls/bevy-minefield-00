use bevy::{
    core::FixedTimestep,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::mouse::MouseButtonInput,
    prelude::*,
    transform,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_prototype_lyon::prelude::*;
use rand::{thread_rng, Rng};

const N: usize = 120;

#[derive(Component)]
struct Rotate;

#[derive(Clone, Copy, Debug)]
enum TileType {
    Grass,
    Dirt,
}

struct Board {
    tiles: Vec<Vec<TileType>>,
}

#[derive(Component)]
struct Coord {
    i: usize,
    j: usize,
}

impl Board {
    fn new(n: usize, m: usize) -> Self {
        let mut rng = thread_rng();
        let mut tiles = vec![vec![TileType::Dirt; n]; m];
        for v in tiles.iter_mut() {
            for t in v.iter_mut() {
                *t = if rng.gen_bool(0.6) {
                    TileType::Grass
                } else {
                    TileType::Dirt
                }
            }
        }
        Board { tiles }
    }
}

fn main() {
    App::new()
        .insert_resource(Board::new(N, N))
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        //.add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(ShapePlugin)
        .add_system(bevy::input::system::exit_on_esc_system)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_startup_system(spawn_entities)
        .add_system(randomize_pos)
        .add_system(handle_mouse)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::steps_per_second(30.))
                .with_system(update_board)
                .with_system(draw_tile),
        )
        .run();
}

fn spawn_entities(mut cmds: Commands) {
    let circle_shape = shapes::Circle {
        radius: 30.0,
        center: Vec2::default(),
    };

    cmds.spawn_bundle(OrthographicCameraBundle::new_2d());

    //cmds.spawn()
    //    .insert_bundle(SpriteBundle {
    //        sprite: Sprite {
    //            color: Color::RED,
    //            custom_size: Some(Vec2::splat(50.)),
    //            ..Default::default()
    //        },
    //        transform: Transform::from_xyz(0., 0., 0.),
    //        ..Default::default()
    //    })
    //    .insert(Rotate)
    //    .with_children(|parent| {
    //        parent.spawn_bundle(GeometryBuilder::build_as(
    //            &circle_shape,
    //            DrawMode::Outlined {
    //                fill_mode: FillMode::color(Color::MIDNIGHT_BLUE),
    //                outline_mode: StrokeMode::new(Color::ALICE_BLUE, 2.0),
    //            },
    //            //Transform::from_xyz(0.0, 0.0, 2.0),
    //            Transform::default(),
    //        ));
    //    });

    //let shape = shapes::Line(Vec2::new(-20., -20.), Vec2::new(50., 50.));
    //cmds.spawn_bundle(GeometryBuilder::build_as(
    //    &shape,
    //    DrawMode::Outlined {
    //        fill_mode: FillMode::color(Color::GREEN),
    //        outline_mode: StrokeMode::new(Color::ALICE_BLUE, 10.0),
    //    },
    //    Transform::default(),
    //));

    //cmds.spawn_bundle(UiCameraBundle::default());
    //cmds.spawn_bundle(ButtonBundle {
    //    style: Style {
    //        size: Size::new(Val::Px(200.), Val::Px(80.)),
    //        margin: Rect::all(Val::Auto),
    //        justify_content: JustifyContent::Center,
    //        align_items: AlignItems::Center,
    //        ..Default::default()
    //    },
    //    color: Color::GRAY.into(),
    //    ..Default::default()
    //});

    let sz = 5.;
    for i in 0..N {
        for j in 0..N {
            cmds.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::BLUE,
                    custom_size: Some(Vec2::splat(sz)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(
                    i as f32 * (sz + 0.1) - (sz * N as f32 * 0.5),
                    j as f32 * (sz + 0.1) - (sz * N as f32 * 0.5),
                    0.2,
                ),
                ..Default::default()
            })
            .insert(Coord { i, j });
        }
    }
}

fn randomize_pos(mut query: Query<&mut Transform, With<Rotate>>, time: Res<Time>) {
    let secs = time.seconds_since_startup() * 3.0;
    for mut transform in query.iter_mut() {
        transform.translation.x = secs.sin() as f32 * 150.;
        transform.translation.y = secs.cos() as f32 * 180.;
    }
}

fn draw_tile(mut query: Query<(&mut Sprite, &Coord)>, board: Res<Board>) {
    let mut rng = thread_rng();
    for (mut sprite, coord) in query.iter_mut() {
        let kind = board.tiles[coord.i][coord.j];
        let col = match kind {
            TileType::Grass => Color::WHITE,
            TileType::Dirt => Color::DARK_GRAY,
            //TileType::Grass => Color::rgb(0.3, 0.8, 0.45),
            //TileType::Dirt => Color::rgb(0.42, 0.35, 0.32),

            //TileType::Grass => Color::rgba(0.3, 0.8, 0.45, rng.gen_range(0.2..0.8)),
            //TileType::Dirt => Color::rgba(0.42, 0.35, 0.32, rng.gen_range(0.2..0.8)),
        };
        sprite.color = col;
    }
}

fn handle_mouse(mouse: Res<Input<MouseButton>>, windows: Res<Windows>, mut cmds: Commands) {
    let win = windows.get_primary().unwrap();
    if mouse.just_pressed(MouseButton::Left) {
        let pos = win.cursor_position().unwrap();
        println!("{pos:?}");

        cmds.insert_resource(Board::new(N, N))
    }
}

fn update_board(mut board: ResMut<Board>) {
    let (n, m) = (board.tiles.len(), board.tiles[0].len());

    let old = board.tiles.clone();
    let mut next = Vec::new();

    let valid = |i: i32, j: i32| i > 0 && j > 0 && i < n as i32 && j < n as i32;

    for i in 0..n {
        for j in 0..m {
            let mut around = Vec::new();

            for u in -1i32..=1 {
                for v in -1i32..=1 {
                    if (u != 0 || v != 0) && valid(u + i as i32, v + j as i32) {
                        around.push(board.tiles[u as usize + i][v as usize + j]);
                    }
                }
            }

            let n: u32 = around
                .iter()
                .map(|t| match t {
                    &TileType::Dirt => 0,
                    &TileType::Grass => 1,
                })
                .sum();

            let ntile = match n {
                0 | 1 => TileType::Dirt,
                2 => old[i][j],
                3 => TileType::Grass,
                4 | 5 | 6 | 7 | 8 => TileType::Dirt,
                _ => unreachable!(),
            };

            next.push((i, j, ntile));
        }
    }

    for (i, j, v) in next {
        board.tiles[i][j] = v;
    }
}
