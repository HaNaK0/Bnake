use bevy::prelude::*;

fn main() {
    App::build()
        .add_startup_system(setup.system())
        .add_plugins(DefaultPlugins)
        .run();   
}

fn setup(
    mut commands: Commands
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
