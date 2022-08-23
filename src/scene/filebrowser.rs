use crate::ui::{icon, EditorTab, Style};
use bevy::asset::AssetIo;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use std::ffi::OsStr;
use std::path::Path;

#[derive(Default, Component)]
pub struct FileBrowser;

impl EditorTab for FileBrowser {
    type Param = (SRes<Style>, SRes<AssetServer>);

    fn ui<'w>(
        &mut self,
        ui: &mut egui::Ui,
        _entity: Entity,
        (style, assets): &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, style.panel);
        let io = assets.asset_io();
        read_dir(ui, io, ".".as_ref());
    }
}

fn read_dir(ui: &mut egui::Ui, io: &dyn AssetIo, path: &Path) {
    ui.indent(&path, |ui| {
        if let Ok(dir) = io.read_directory(path) {
            for path in dir {
                let path_string = path.file_name().unwrap().to_string_lossy().to_string();
                if io.is_dir(&path) {
                    ui.label(format!("{} {}", icon::FILE_FOLDER, path_string));
                    read_dir(ui, io, &path);
                } else {
                    let ext = path.extension().map(OsStr::to_string_lossy);
                    let icon = if let Some(ext) = ext.as_deref() {
                        match ext {
                            "scene" => icon::SCENE_DATA,
                            "material" => icon::MATERIAL_DATA,
                            "image" => icon::TEXTURE_DATA,
                            "mesh" => icon::MESH_DATA,

                            "shader" => icon::NODE_MATERIAL,

                            "gltf" | "glb" => icon::FILE_3D,
                            "jpg" | "jpeg" | "png" => icon::FILE_IMAGE,
                            "mp3" | "ogg" | "flac" | "wav" => icon::FILE_SOUND,

                            _ => icon::FILE,
                        }
                    } else {
                        icon::FILE
                    };

                    ui.label(format!("{} {}", icon, path_string));
                }
            }
        }
    });
}
