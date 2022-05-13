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
                    ui.label(format!("{} {}", crate::blender::FILE_FOLDER, path_string));
                    read_dir(ui, io, &path);
                } else {
                    let ext = path.extension().map(OsStr::to_string_lossy);
                    let icon = if let Some(ext) = ext.as_deref() {
                        match ext {
                            "scene" => crate::blender::SCENE_DATA,
                            "material" => crate::blender::MATERIAL_DATA,
                            "image" => crate::blender::TEXTURE_DATA,
                            "mesh" => crate::blender::MESH_DATA,

                            "shader" => crate::blender::NODE_MATERIAL,

                            "gltf" | "glb" => crate::blender::FILE_3D,
                            "jpg" | "jpeg" | "png" => crate::blender::FILE_IMAGE,
                            "mp3" | "ogg" | "flac" | "wav" => crate::blender::FILE_SOUND,

                            _ => crate::blender::FILE,
                        }
                    } else {
                        crate::blender::FILE
                    };

                    ui.label(format!("{} {}", icon, path_string));
                }
            }
        }
    });
}
