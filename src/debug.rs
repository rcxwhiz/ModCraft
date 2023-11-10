use bevy::{prelude::*, render::render_resource::{Extent3d, TextureDimension, TextureFormat}};

#[derive(Resource)]
pub struct UVDebugTexture(pub Image);

impl Default for UVDebugTexture {
    fn default() -> Self {
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

        Self(Image::new_fill(
            Extent3d {
                width: TEXTURE_SIZE as u32,
                height: TEXTURE_SIZE as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &texture_data,
            TextureFormat::Rgba8UnormSrgb,
        ))
    }
}

fn insert_uv_debug_texture(mut commands: Commands) {
    info!("Adding debug texture resource");
    commands.init_resource::<UVDebugTexture>();
    info!("Added debug texture resource");
}

#[derive(Resource)]
pub struct CubeMesh(pub Handle<Mesh>);

fn insert_cube_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    info!("Adding cube mesh resource");
    let cube_mesh = CubeMesh(meshes.add(shape::Cube::default().into()));
    commands.insert_resource(cube_mesh);
    info!("Added cube mesh resource");
}

#[derive(Resource)]
pub struct DebugMaterial(pub Handle<StandardMaterial>);

fn insert_debug_material(
    mut commands: Commands,
    uv_debug_texture: Res<UVDebugTexture>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Adding debug material resource");
    let debug_material = DebugMaterial(materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture.0.clone())),
        ..default()
    }));

    commands.insert_resource(debug_material);
    info!("Added debug material resource");
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, insert_uv_debug_texture);
        app.add_systems(Startup, (insert_cube_mesh, insert_debug_material));
    }
}
