pub mod vanilla;

use crate::game::Game;

use vanilla::VanillaLoader;

pub fn minecraft() -> Game {
    let mut minecraft = Game::new("minecraft".to_string());
    minecraft.add_loader(VanillaLoader::default());
    minecraft
}
