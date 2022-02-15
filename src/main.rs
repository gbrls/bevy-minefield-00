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

use bevy::ecs::schedule::ShouldRun;

const N: usize = 60;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Rotate;

#[derive(Clone, Copy, Debug)]
enum TileType {
    Grass,
    Dirt,
}

struct GameClick {
    pos: Vec2,
}

struct Board {
    tiles: Vec<Vec<TileType>>,
    selected: Option<Coord>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
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
                *t = if rng.gen_bool(0.1) {
                    TileType::Grass
                } else {
                    TileType::Dirt
                }
            }
        }
        Board {
            tiles,
            selected: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum GameState {
    Running,
    Editing,
}

fn main() {
    App::new()
        .insert_resource(Board::new(N, N))
        .add_state(GameState::Running)
        .add_plugins(DefaultPlugins)
        .add_event::<GameClick>()
        //add_plugin(WorldInspectorPlugin::new())
        //.add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(ShapePlugin)
        .add_system(bevy::input::system::exit_on_esc_system)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_startup_system(spawn_entities)
        .add_system(randomize_pos)
        .add_system(handle_mouse_position)
        .add_system(select_tile)
        .add_system(draw_board)
        .add_system(mouse_click)
        .add_system(handle_pause)
        .add_system(update_text)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(
                    FixedTimestep::steps_per_second(20.0).chain(
                        (|In(input): In<ShouldRun>, state: Res<State<GameState>>| {
                            if state.current() == &GameState::Running {
                                input
                            } else {
                                ShouldRun::No
                            }
                        })
                        .system(),
                    ),
                )
                .with_system(update_board.system()),
        )
        .run();
}


fn spawn_entities(mut cmds: Commands, asset_server: Res<AssetServer>) {
    let circle_shape = shapes::Circle {
        radius: 30.0,
        center: Vec2::default(),
    };

    cmds.spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);

    cmds.spawn_bundle(UiCameraBundle::default());
    cmds.spawn_bundle(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(5.0),
                right: Val::Px(15.0),
                ..Default::default()
            },
            ..Default::default()
        },
        text: Text::with_section(
            "Hello, world!",
            TextStyle {
                font: asset_server.load("fonts/ShareTechMono-Regular.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
            TextAlignment {
                horizontal: HorizontalAlign::Center,
                ..Default::default()
            },
        ),
        ..Default::default()
    });

    let sz = 10.;
    for i in 0..N {
        for j in 0..N {
            cmds.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::BLUE,
                    custom_size: Some(Vec2::splat(sz)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(
                    i as f32 * (sz + sz * 0.05) - (sz * N as f32 * 0.5),
                    j as f32 * (sz + sz * 0.05) - (sz * N as f32 * 0.5),
                    0.2,
                ),
                ..Default::default()
            })
            .insert(Coord { i, j });
        }
    }
}

fn update_text(mut query: Query<&mut Text>, state: Res<State<GameState>>) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("{:?}", state.current());
    }
}

fn randomize_pos(mut query: Query<&mut Transform, With<Rotate>>, time: Res<Time>) {
    let secs = time.seconds_since_startup() * 3.0;
    for mut transform in query.iter_mut() {
        transform.translation.x = secs.sin() as f32 * 150.;
        transform.translation.y = secs.cos() as f32 * 180.;
    }
}

fn draw_board(mut query: Query<(&mut Sprite, &Coord)>, board: Res<Board>) {
    for (mut sprite, coord) in query.iter_mut() {
        let eq = if let Some(sel_coord) = board.selected {
            *coord == sel_coord
        } else {
            false
        };
        let kind = board.tiles[coord.i][coord.j];
        let col = match (eq, kind) {
            (true, _) => Color::YELLOW,
            (_, TileType::Grass) => Color::BLUE,
            (_, TileType::Dirt) => Color::WHITE,
            //TileType::Grass => Color::rgb(0.3, 0.8, 0.45),
            //TileType::Dirt => Color::rgb(0.42, 0.35, 0.32),
        };
        sprite.color = col;
    }
}

fn select_tile(
    mut click_event: EventReader<GameClick>,
    mut board: ResMut<Board>,
    query: Query<(&Transform, &Sprite, &Coord)>,
) {
    if let Some(click_event) = click_event.iter().last() {
        for (pos, sprite, coord) in query.iter() {
            let pos = Vec2::new(pos.translation.x, pos.translation.y);
            let d = pos.distance(click_event.pos);

            if let Some(sz) = sprite.custom_size {
                if d <= sz.x * 0.8 {
                    board.selected = Some(*coord);
                }
            }
        }
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

fn handle_mouse_position(
    windows: Res<Windows>,
    camera: Query<&Transform, With<MainCamera>>,
    mut click_event_writer: EventWriter<GameClick>,
) {
    let win = windows.get_primary().unwrap();
    if let Some(pos) = win.cursor_position() {
        let size = Vec2::new(win.width() as f32, win.height() as f32);

        // the default orthographic projection is in pixels from the center;
        // just undo the translation
        let p = pos - size / 2.0;

        // assuming there is exactly one main camera entity, so this is OK
        let camera_transform = camera.single();

        // apply the camera transform
        let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);

        click_event_writer.send(GameClick {
            pos: Vec2::new(pos_wld.x, pos_wld.y),
        });
    }
}

fn mouse_click(mouse: Res<Input<MouseButton>>, mut board: ResMut<Board>) {
    if mouse.pressed(MouseButton::Left) {
        if let Some(cur) = board.selected {
            board.tiles[cur.i][cur.j] = TileType::Grass;
        }
    }

    if mouse.pressed(MouseButton::Right) {
        if let Some(cur) = board.selected {
            board.tiles[cur.i][cur.j] = TileType::Dirt;
        }
    }
}

fn handle_pause(keys: Res<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
    if keys.just_pressed(KeyCode::Space) {
        let next = match state.current() {
            GameState::Running => GameState::Editing,
            GameState::Editing => GameState::Running,
        };

        println!("Space");

        state.set(next).unwrap();
    }
}
