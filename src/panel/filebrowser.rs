use crate::app::EditorPanel;
use crate::style::Style;
use bevy::asset::AssetIo;
use bevy::prelude::*;
use bevy::window::WindowId;
use std::ffi::OsStr;
use std::path::Path;

#[derive(Default, Component)]
pub struct FileBrowser;

impl FileBrowser {
    pub fn system(
        mut context: ResMut<crate::shell::EguiContext>,
        style: Res<Style>,
        query: Query<(Entity, &EditorPanel), With<Self>>,
        assets: Res<AssetServer>,
    ) {
        let [ctx] = context.ctx_mut([WindowId::primary()]);
        for (entity, viewport) in query.iter() {
            if let Some(viewport) = viewport.viewport {
                let id = egui::Id::new("FileBrowser").with(entity);
                let mut ui = egui::Ui::new(
                    ctx.clone(),
                    egui::LayerId::background(),
                    id,
                    viewport,
                    viewport,
                );

                let rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(rect, 0.0, style.panel);

                let io = assets.asset_io();

                read_dir(&mut ui, io, ".".as_ref());
            }
        }
    }
}

fn read_dir(ui: &mut egui::Ui, io: &dyn AssetIo, path: &Path) {
    ui.indent(&path, |ui| {
        if let Ok(dir) = io.read_directory(path) {
            for path in dir {
                let path_string = path.file_name().unwrap().to_string_lossy().to_string();
                if io.is_dir(&path) {
                    ui.label(format!("{} {}", crate::icon::FILE_FOLDER, path_string));
                    read_dir(ui, io, &path);
                } else {
                    let ext = path.extension().map(OsStr::to_string_lossy);
                    let icon = if let Some(ext) = ext.as_deref() {
                        match ext {
                            "scene" => crate::icon::SCENE_DATA,
                            "material" => crate::icon::MATERIAL_DATA,
                            "image" => crate::icon::TEXTURE_DATA,
                            "mesh" => crate::icon::MESH_DATA,

                            "shader" => crate::icon::NODE_MATERIAL,

                            "gltf" | "glb" => crate::icon::FILE_3D,
                            "jpg" | "jpeg" | "png" => crate::icon::FILE_IMAGE,
                            "mp3" | "ogg" | "flac" | "wav" => crate::icon::FILE_SOUND,

                            _ => crate::icon::FILE,
                        }
                    } else {
                        crate::icon::FILE
                    };

                    ui.label(format!("{} {}", icon, path_string));
                }
            }
        }
    });
}
