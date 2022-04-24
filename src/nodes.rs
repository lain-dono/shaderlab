pub mod builtin;
pub mod channel;
pub mod input;
pub mod logic;
pub mod master;
pub mod math;
pub mod util;
pub mod uv;

fn label(label: &str) -> egui::RichText {
    if let Some(label) = label.strip_prefix('-') {
        egui::RichText::new(label)
            .strikethrough()
            .color(egui::Color32::RED)
    } else {
        egui::RichText::new(label).color(egui::Color32::WHITE)
    }
}

fn col(ui: &mut egui::Ui, title: &str, children: &[&str]) {
    egui::CollapsingHeader::new(label(title))
        .id_source((title, "##########"))
        .default_open(true)
        .show(ui, |ui| {
            for &child in children {
                ui.label(label(child));
            }
        });
}

fn col_horizontal(ui: &mut egui::Ui, title: &str, children: &[&str]) {
    egui::CollapsingHeader::new(title)
        .id_source((title, "##########"))
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for &child in children {
                    ui.label(label(child));
                }
            });
        });
}

pub fn nodes(ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .id_source("__########__")
        .always_show_scroll(true)
        .show(ui, |ui| {
            ui.columns(4, |columns| {
                columns[0].scope(artistic);
                columns[1].scope(input);
                columns[2].scope(math);
                columns[3].scope(procedural);
            });
        });
}

fn artistic(ui: &mut egui::Ui) {
    ui.heading("Artistic");
    col(
        ui,
        "Adjustment",
        &[
            "Channel Mixer",
            "Contrast",
            "Hue",
            "Invert Colors",
            "Replace Color",
            "Saturation",
            "White Balance",
        ],
    );
    col(ui, "Blend", &["Blend"]);
    col(ui, "Filter", &["Dither"]);
    col(ui, "Mask", &["Channel Mask", "Color Mask"]);
    col(
        ui,
        "Normal",
        &[
            "Normal Blend",
            "Normal From Height",
            "Normal From Texture",
            "Normal Reconstruct Z",
            "Normal Strength",
            "Normal Unpack",
        ],
    );
    col(
        ui,
        "Utility",
        &["Colorspace Conversion", "Sub Graph Dropdown"],
    );

    ui.heading("Procedural");

    ui.label("Checkerboard");
    col(
        ui,
        "Noise",
        &["-Gradient Noise", "-Simple Noise", "Voronoi"],
    );
    col(
        ui,
        "Shapes",
        &[
            "Ellipse",
            "Polygon",
            "Rectangle",
            "Rounded Polygon",
            "Rounded Rectangle",
        ],
    );
}

fn input(ui: &mut egui::Ui) {
    ui.heading("Input");
    col_horizontal(
        ui,
        "Basic ",
        &[
            "-Boolean",
            "-Color",
            "-Constant",
            "Integer",
            "-Slider",
            "Time",
            "-Float",
            "-Vector 2",
            "-Vector 3",
            "-Vector 4",
        ],
    );
    col_horizontal(
        ui,
        "Geometry",
        &[
            "Bitangent Vector",
            "Instance ID",
            "Normal Vector",
            "Position",
            "Screen Position",
            "Tangent Vector",
            "UV",
            "Vertex Color",
            "Vertex ID",
            "View Direction",
            "View Vector",
        ],
    );
    col_horizontal(
        ui,
        "Gradient",
        &["-Blackbody", "Gradient", "Sample Gradient"],
    );
    col_horizontal(
        ui,
        "HDRP",
        &["Diffusion Profile", "Exposure", "HD Scene Color"],
    );
    col_horizontal(
        ui,
        "Lighting",
        &[
            "Ambient",
            "Baked GI",
            "Main Light Direction",
            "Reflection Probe",
        ],
    );
    col_horizontal(
        ui,
        "Matrix ",
        &[
            "Matrix 2x2",
            "Matrix 3x3",
            "Matrix 4x4",
            "Transformation Matrix",
        ],
    );
    col(
        ui,
        "Mesh Deformation",
        &["Compute Deformation", "Linear Blend Skinning"],
    );
    col(ui, "PBR", &["Dielectric Specular", "Metal Reflectance"]);
    col_horizontal(
        ui,
        "Scene",
        &[
            "Camera",
            "Eye Index",
            "Fog",
            "Object",
            "Scene Color",
            "Scene Depth",
            "Screen",
        ],
    );
    col(
        ui,
        "Texture",
        &[
            "Calculate Level Of Detail Texture 2D Node",
            "Cubemap Asset",
            "Gather Texture 2D Node",
            "Sample Cubemap",
            "Sample Reflected Cubemap",
            "Sample Texture 2D",
            "Sample Texture 2D Array",
            "Sample Texture 2D LOD",
            "Sample Texture 3D",
            "Sample Virtual Texture",
            "Sampler State",
            "Split Texture Transform",
            "Texel Size",
            "Texture 2D Array Asset",
            "Texture 2D Asset",
            "Texture 3D Asset",
        ],
    );
}

fn math(ui: &mut egui::Ui) {
    ui.heading("Math");
    col_horizontal(
        ui,
        "Advanced",
        &[
            "-Absolute",
            "-Exponential",
            "-Length",
            "-Log",
            "-Modulo",
            "-Negate",
            "-Normalize",
            "-Posterize",
            "Reciprocal",
            "-Reciprocal Square Root",
        ],
    );

    col_horizontal(
        ui,
        "Basic",
        &[
            "-Add",
            "-Divide",
            "-Multiply",
            "-Power",
            "-Square Root",
            "-Subtract",
        ],
    );

    col_horizontal(ui, "Derivative", &["-DDX", "-DDXY", "-DDY"]);
    col_horizontal(
        ui,
        "Interpolation",
        &["Inverse Lerp", "-Lerp(Mix)", "-Smoothstep"],
    );
    col(
        ui,
        "Matrix",
        &[
            "Matrix Construction",
            "-Matrix Determinant",
            "Matrix Split",
            "-Matrix Transpose",
        ],
    );
    col_horizontal(
        ui,
        "Range",
        &[
            "-Clamp",
            "-Fraction",
            "-Maximum",
            "-Minimum",
            "One Minus",
            "Random Range",
            "-Remap",
            "Saturate", // TODO: maybe clamp with defaults
        ],
    );
    col_horizontal(
        ui,
        "Round",
        &[
            "-Ceiling",
            "-Floor",
            "-Round",
            "-Sign",
            "-Step",
            "-Truncate",
        ],
    );
    col_horizontal(
        ui,
        "Trigonometry",
        &[
            "-Arccosine",
            "-Arcsine",
            "-Arctangent",
            "-Arctangent2",
            "-Cosine",
            "Degrees To Radians",
            "-Hyperbolic Cosine",
            "-Hyperbolic Sine",
            "-Hyperbolic Tangent",
            "Radians To Degrees",
            "-Sine",
            "-Tangent",
        ],
    );
    col_horizontal(
        ui,
        "Vector",
        &[
            "-Cross Product",
            "-Distance",
            "-Dot Product",
            "Fresnel Effect",
            "Projection",
            "-Reflection",
            "Rejection",
            "Rotate About Axis",
            "Sphere Mask",
            "Transform",
        ],
    );
    col_horizontal(
        ui,
        "Wave",
        &[
            "Noise Sine Wave",
            "Sawtooth Wave",
            "Square Wave",
            "Triangle Wave",
        ],
    );
}

fn procedural(ui: &mut egui::Ui) {
    ui.heading("Utility");

    ui.label("Custom Function");
    ui.label("Keyword");
    ui.label("Preview");
    ui.label("Sub Graph");

    ui.label("HDRP Emission");

    col_horizontal(
        ui,
        "Logic",
        &[
            "All",
            "And",
            "Any",
            "-Branch",
            "-Comparison",
            "Is Front Face",
            "Is Infinite",
            "Is NaN",
            "Nand",
            "-Not",
            "Or",
        ],
    );
    /*
    col(
        ui,
        "Eye",
        &[
            "Circle Pupil Animation",
            "Cornea Refraction",
            "Eye Surface Type Debug",
            "Iris Limbal Ring",
            "Iris Offset",
            "Iris Out Of Bound Color Clamp",
            "Iris UVLocation",
            "Sclera Iris Blend",
            "Sclera Limbal Ring",
            "Sclera UVLocation",
        ],
    );
    */

    col(
        ui,
        "UV",
        &[
            "-Flipbook",
            "Polar Coordinates",
            "Radial Shear",
            "Rotate",
            "Spherize",
            "Tiling And Offset",
            "Triplanar",
            "Twirl",
            "Parallax Mapping",
            "Parallax Occlusion Mapping",
        ],
    );
    //col(ui, "Block Nodes", &["Built In Blocks"]);

    col(
        ui,
        "Channel",
        &[
            "-Combine",
            "Flip",
            "-Split",
            "Swizzle",
            "Branch On Input Connection",
        ],
    );
}
