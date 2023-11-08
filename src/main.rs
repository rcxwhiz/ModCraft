use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // spawn camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..Default::default()
    });

    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let cube_mesh = meshes.add(shape::Cube::default().into());

    // spawn a cube
    // commands.spawn(PbrBundle {
    //     mesh: cube_mesh.clone(),
    //     transform: Transform::from_xyz(10.0, 0.0, 0.0),
    //     material: debug_material.clone(),
    //     ..Default::default()
    // });

    // commands.spawn(PbrBundle {
    //     mesh: cube_mesh.clone(),
    //     transform: Transform::from_xyz(-10.0, 0.0, 0.0),
    //     material: debug_material.clone(),
    //     ..Default::default()
    // });

    // commands.spawn(PbrBundle {
    //     mesh: cube_mesh.clone(),
    //     transform: Transform::from_xyz(0.0, 0.0, -10.0),
    //     material: debug_material.clone(),
    //     ..Default::default()
    // });

    commands.spawn(PbrBundle {
        mesh: cube_mesh.clone(),
        transform: Transform::from_xyz(0.0, 10.0, 0.0),
        material: debug_material.clone(),
        ..Default::default()
    });

    // commands.spawn(PbrBundle {
    //     mesh: cube_mesh.clone(),
    //     transform: Transform::from_xyz(0.0, -10.0, 0.0),
    //     material: debug_material.clone(),
    //     ..Default::default()
    // });

    // commands.spawn(PbrBundle {
    //     mesh: cube_mesh,
    //     transform: Transform::from_xyz(0.0, 0.0, 10.0),
    //     material: debug_material,
    //     ..Default::default()
    // });
}

// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}
