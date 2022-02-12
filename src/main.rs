use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
enum TileType {
    Grass { is_bomb: bool },
    Dirt,
}

struct Board {
    tiles: Vec<Vec<TileType>>,
}

impl Board {
    fn new(n: usize, m: usize) -> Self {
        Board {
            tiles: vec![vec![TileType::Dirt; n]; m],
        }
    }
}

fn main() {
    App::new()
        .insert_resource(Board::new(10, 10))
        .add_plugins(DefaultPlugins)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn print_tiles(query: Query<&TileType>) {
    for tile in query.iter() {
        println!("{tile:?}");
    }
}
