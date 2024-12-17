use bevy::prelude::*;
use bevy_wasm::prelude::*;
use cubes_protocol::{HostMessage, ModMessage, PROTOCOL_VERSION};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WasmPlugin::<HostMessage, ModMessage>::new(PROTOCOL_VERSION))
        .add_systems(Startup, insert_mods)
        .add_systems(Startup, setup)
        .add_systems(Update, update_cubes_from_mods)
        .run();
}

fn insert_mods(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(WasmMod {
        wasm: asset_server.load("mod_with_bevy.wasm"),
    });
    commands.spawn(WasmMod {
        wasm: asset_server.load("mod_without_bevy.wasm"),
    });
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(
            Plane3d::default().mesh().size(5.0, 5.0).subdivisions(0),
        ))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));
    commands.spawn((
        PointLight {
            intensity: 1000000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0)
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 3.5, 5.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y)
    ));
}

fn update_cubes_from_mods(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mod_messages: EventReader<ModMessage>,
    mut host_messages: EventWriter<HostMessage>,
    mut query: Query<&mut Transform>,
) {
    for event in mod_messages.read() {
        match event {
            ModMessage::MoveCube { entity_id, x, y, z } => {
                if let Ok(mut transform) = query.get_mut(Entity::from_raw(*entity_id)) {
                    transform.translation = Vec3::new(*x, *y, *z);
                }
            }
            ModMessage::SpawnCube { mod_state, color } => {
                info!("Spawning cube from mod {:x}!", mod_state);
                let entity_id = commands
                    .spawn((
                        Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.5, 0.5, 0.5)))),
                        MeshMaterial3d(materials.add(Color::srgb(color.0, color.1, color.2))),
                        Transform::from_xyz(0.0, 0.5, 0.0)
                    ))
                    .id()
                    .index();
                host_messages.send(HostMessage::SpawnedCube {
                    mod_state: *mod_state,
                    entity_id,
                });
            }
        }
    }
}
