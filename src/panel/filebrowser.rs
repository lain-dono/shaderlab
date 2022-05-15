use crate::app::TabInner;
use crate::context::EditorContext;
use crate::style::Style;
use bevy::asset::AssetIo;
use std::ffi::OsStr;
use std::path::Path;

#[derive(Default)]
pub struct FileBrowser;

impl TabInner for FileBrowser {
    fn ui(&mut self, ui: &mut egui::Ui, style: &Style, ctx: EditorContext) {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, style.panel);

        let io = ctx.assets.asset_io();

        read_dir(ui, io, ".".as_ref());
    }
}

fn read_dir(ui: &mut egui::Ui, io: &dyn AssetIo, path: &Path) {
    ui.indent(&path, |ui| {
        if let Ok(dir) = io.read_directory(path) {
            for path in dir {
                let path_string = path.file_name().unwrap().to_string_lossy().to_string();
                if io.is_directory(&path) {
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
