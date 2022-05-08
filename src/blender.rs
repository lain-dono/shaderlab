#![allow(dead_code)]

pub const ACTION: char = '\u{e900}';
pub const ACTION_TWEAK: char = '\u{e901}';
pub const ADD: char = '\u{e902}';
pub const ALIASED: char = '\u{e903}';
pub const ALIGN_BOTTOM: char = '\u{e904}';
pub const ALIGN_CENTER: char = '\u{e905}';
pub const ALIGN_FLUSH: char = '\u{e906}';
pub const ALIGN_JUSTIFY: char = '\u{e907}';
pub const ALIGN_LEFT: char = '\u{e908}';
pub const ALIGN_MIDDLE: char = '\u{e909}';
pub const ALIGN_RIGHT: char = '\u{e90a}';
pub const ALIGN_TOP: char = '\u{e90b}';
pub const ANCHOR_BOTTOM: char = '\u{e90c}';
pub const ANCHOR_CENTER: char = '\u{e90d}';
pub const ANCHOR_LEFT: char = '\u{e90e}';
pub const ANCHOR_RIGHT: char = '\u{e90f}';
pub const ANCHOR_TOP: char = '\u{e910}';
pub const ANIM: char = '\u{e911}';
pub const ANIM_DATA: char = '\u{e912}';
pub const ANTIALIASED: char = '\u{e913}';
pub const APPEND_BLEND: char = '\u{e914}';
pub const ARMATURE_DATA: char = '\u{e915}';
pub const ARROW_LEFTRIGHT: char = '\u{e916}';
pub const ASSET_MANAGER: char = '\u{e917}';
pub const AUTO: char = '\u{e918}';
pub const AUTOMERGE_OFF: char = '\u{e919}';
pub const AUTOMERGE_ON: char = '\u{e91a}';
pub const AXIS_FRONT: char = '\u{e91b}';
pub const AXIS_SIDE: char = '\u{e91c}';
pub const AXIS_TOP: char = '\u{e91d}';
pub const BACK: char = '\u{e91e}';
pub const BLENDER: char = '\u{e91f}';
pub const BOIDS: char = '\u{e920}';
pub const BOLD: char = '\u{e921}';
pub const BONE_DATA: char = '\u{e922}';
pub const BOOKMARKS: char = '\u{e923}';
pub const BORDERMOVE: char = '\u{e924}';
pub const BRUSH_DATA: char = '\u{e925}';
pub const BRUSHES_ALL: char = '\u{e926}';
pub const CAMERA_DATA: char = '\u{e927}';
pub const CAMERA_STEREO: char = '\u{e928}';
pub const CANCEL: char = '\u{e929}';
pub const CENTER_ONLY: char = '\u{e92a}';
pub const CHECKBOX_DEHLT: char = '\u{e92b}';
pub const CHECKBOX_HLT: char = '\u{e92c}';
pub const CHECKMARK: char = '\u{e92d}';
pub const CLIP_UV_DEHLT: char = '\u{e92e}';
pub const CLIP_UV_HLT: char = '\u{e92f}';
pub const COLLAPSEMENU: char = '\u{e930}';
pub const COLLECTION_NEW: char = '\u{e931}';
pub const COLOR: char = '\u{e932}';
pub const COLOR_ALPHA: char = '\u{e933}';
pub const COLOR_BLUE: char = '\u{e934}';
pub const COLOR_GREEN: char = '\u{e935}';
pub const COLOR_RED: char = '\u{e936}';
pub const COMMUNITY: char = '\u{e937}';
pub const CON_ACTION: char = '\u{e938}';
pub const CON_ARMATURE: char = '\u{e939}';
pub const CON_CAMERASOLVER: char = '\u{e93a}';
pub const CON_CHILDOF: char = '\u{e93b}';
pub const CON_CLAMPTO: char = '\u{e93c}';
pub const CON_DISTLIMIT: char = '\u{e93d}';
pub const CONE: char = '\u{e93e}';
pub const CON_FLOOR: char = '\u{e93f}';
pub const CON_FOLLOWPATH: char = '\u{e940}';
pub const CON_FOLLOWTRACK: char = '\u{e941}';
pub const CON_KINEMATIC: char = '\u{e942}';
pub const CON_LOCKTRACK: char = '\u{e943}';
pub const CON_LOCLIKE: char = '\u{e944}';
pub const CON_LOCLIMIT: char = '\u{e945}';
pub const CON_OBJECTSOLVER: char = '\u{e946}';
pub const CON_PIVOT: char = '\u{e947}';
pub const CON_ROTLIKE: char = '\u{e948}';
pub const CON_ROTLIMIT: char = '\u{e949}';
pub const CON_SAMEVOL: char = '\u{e94a}';
pub const CON_SHRINKWRAP: char = '\u{e94b}';
pub const CON_SIZELIKE: char = '\u{e94c}';
pub const CON_SIZELIMIT: char = '\u{e94d}';
pub const CONSOLE: char = '\u{e94e}';
pub const CON_SPLINEIK: char = '\u{e94f}';
pub const CONSTRAINT: char = '\u{e950}';
pub const CONSTRAINT_BONE: char = '\u{e951}';
pub const CON_STRETCHTO: char = '\u{e952}';
pub const CON_TRACKTO: char = '\u{e953}';
pub const CON_TRANSFORM: char = '\u{e954}';
pub const CON_TRANSFORM_CACHE: char = '\u{e955}';
pub const CON_TRANSLIKE: char = '\u{e956}';
pub const COPY_DOWN: char = '\u{e957}';
pub const COPY_ID: char = '\u{e958}';
pub const CUBE: char = '\u{e959}';
pub const CURSOR: char = '\u{e95a}';
pub const CURVE_BEZCIRCLE: char = '\u{e95b}';
pub const CURVE_BEZCURVE: char = '\u{e95c}';
pub const CURVE_DATA: char = '\u{e95d}';
pub const CURVE_NCIRCLE: char = '\u{e95e}';
pub const CURVE_NCURVE: char = '\u{e95f}';
pub const CURVE_PATH: char = '\u{e960}';
pub const DECORATE: char = '\u{e961}';
pub const DECORATE_ANIMATE: char = '\u{e962}';
pub const DECORATE_DRIVER: char = '\u{e963}';
pub const DECORATE_KEYFRAME: char = '\u{e964}';
pub const DECORATE_LIBRARY_OVERRIDE: char = '\u{e965}';
pub const DECORATE_LINKED: char = '\u{e966}';
pub const DECORATE_LOCKED: char = '\u{e967}';
pub const DECORATE_OVERRIDE: char = '\u{e968}';
pub const DECORATE_UNLOCKED: char = '\u{e969}';
pub const DESKTOP: char = '\u{e96a}';
pub const DISCLOSURE_TRI_DOWN: char = '\u{e96b}';
pub const DISCLOSURE_TRI_RIGHT: char = '\u{e96c}';
pub const DISK: char = '\u{e96d}';
pub const DISK_DRIVE: char = '\u{e96e}';
pub const DOCUMENTS: char = '\u{e96f}';
pub const DOT: char = '\u{e970}';
pub const DOWNARROW_HLT: char = '\u{e971}';
pub const DRIVER: char = '\u{e972}';
pub const DRIVER_DISTANCE: char = '\u{e973}';
pub const DRIVER_ROTATIONAL_DIFFERENCE: char = '\u{e974}';
pub const DRIVER_TRANSFORM: char = '\u{e975}';
pub const DUPLICATE: char = '\u{e976}';
pub const EDGE_SELECT: char = '\u{e977}';
pub const EDITMODE_HLT: char = '\u{e978}';
pub const EMPTY_ARROWS: char = '\u{e979}';
pub const EMPTY_AXIS: char = '\u{e97a}';
pub const EMPTY_DATA: char = '\u{e97b}';
pub const EMPTY_SINGLE_ARROW: char = '\u{e97c}';
pub const ERROR: char = '\u{e97d}';
pub const EXPERIMENTAL: char = '\u{e97e}';
pub const EXTERNAL_DRIVE: char = '\u{e97f}';
pub const EYEDROPPER: char = '\u{e980}';
pub const FACE_MAPS: char = '\u{e981}';
pub const FACE_SELECT: char = '\u{e982}';
pub const FAKE_USER_OFF: char = '\u{e983}';
pub const FAKE_USER_ON: char = '\u{e984}';
pub const FCURVE: char = '\u{e985}';
pub const FCURVE_SNAPSHOT: char = '\u{e986}';
pub const FF: char = '\u{e987}';
pub const FILE: char = '\u{e988}';
pub const FILE_3D: char = '\u{e989}';
pub const FILE_ARCHIVE: char = '\u{e98a}';
pub const FILE_BACKUP: char = '\u{e98b}';
pub const FILE_BLANK: char = '\u{e98c}';
pub const FILE_BLEND: char = '\u{e98d}';
pub const FILEBROWSER: char = '\u{e98e}';
pub const FILE_CACHE: char = '\u{e98f}';
pub const FILE_FOLDER: char = '\u{e990}';
pub const FILE_FONT: char = '\u{e991}';
pub const FILE_HIDDEN: char = '\u{e992}';
pub const FILE_IMAGE: char = '\u{e993}';
pub const FILE_MOVIE: char = '\u{e994}';
pub const FILE_NEW: char = '\u{e995}';
pub const FILE_PARENT: char = '\u{e996}';
pub const FILE_REFRESH: char = '\u{e997}';
pub const FILE_SCRIPT: char = '\u{e998}';
pub const FILE_SOUND: char = '\u{e999}';
pub const FILE_TEXT: char = '\u{e99a}';
pub const FILE_TICK: char = '\u{e99b}';
pub const FILE_VOLUME: char = '\u{e99c}';
pub const FILTER: char = '\u{e99d}';
pub const FOLDER_REDIRECT: char = '\u{e99e}';
pub const FONT_DATA: char = '\u{e99f}';
pub const FONT_PREVIEW: char = '\u{e9a0}';
pub const FORCE_BOID: char = '\u{e9a1}';
pub const FORCE_CHARGE: char = '\u{e9a2}';
pub const FORCE_CURVE: char = '\u{e9a3}';
pub const FORCE_DRAG: char = '\u{e9a4}';
pub const FORCE_FLUIDFLOW: char = '\u{e9a5}';
pub const FORCE_FORCE: char = '\u{e9a6}';
pub const FORCE_HARMONIC: char = '\u{e9a7}';
pub const FORCE_LENNARDJONES: char = '\u{e9a8}';
pub const FORCE_MAGNETIC: char = '\u{e9a9}';
pub const FORCE_TEXTURE: char = '\u{e9aa}';
pub const FORCE_TURBULENCE: char = '\u{e9ab}';
pub const FORCE_VORTEX: char = '\u{e9ac}';
pub const FORCE_WIND: char = '\u{e9ad}';
pub const FORWARD: char = '\u{e9ae}';
pub const FRAME_NEXT: char = '\u{e9af}';
pub const FRAME_PREV: char = '\u{e9b0}';
pub const FREEZE: char = '\u{e9b1}';
pub const FULLSCREEN_ENTER: char = '\u{e9b2}';
pub const FULLSCREEN_EXIT: char = '\u{e9b3}';
pub const FUND: char = '\u{e9b4}';
pub const GHOST_DISABLED: char = '\u{e9b5}';
pub const GHOST_ENABLED: char = '\u{e9b6}';
pub const GIZMO: char = '\u{e9b7}';
pub const GP_MULTIFRAME_EDITING: char = '\u{e9b8}';
pub const GP_ONLY_SELECTED: char = '\u{e9b9}';
pub const GP_SELECT_BETWEEN_STROKES: char = '\u{e9ba}';
pub const GP_SELECT_POINTS: char = '\u{e9bb}';
pub const GP_SELECT_STROKES: char = '\u{e9bc}';
pub const GRAPH: char = '\u{e9bd}';
pub const GREASEPENCIL: char = '\u{e9be}';
pub const GRID: char = '\u{e9bf}';
pub const GRIP: char = '\u{e9c0}';
pub const GROUP: char = '\u{e9c1}';
pub const GROUP_BONE: char = '\u{e9c2}';
pub const GROUP_UVS: char = '\u{e9c3}';
pub const GROUP_VCOL: char = '\u{e9c4}';
pub const GROUP_VERTEX: char = '\u{e9c5}';
pub const HAIR: char = '\u{e9c6}';
pub const HAIR_DATA: char = '\u{e9c7}';
pub const HAND: char = '\u{e9c8}';
pub const HANDLE_ALIGNED: char = '\u{e9c9}';
pub const HANDLE_AUTO: char = '\u{e9ca}';
pub const HANDLE_AUTOCLAMPED: char = '\u{e9cb}';
pub const HANDLE_FREE: char = '\u{e9cc}';
pub const HANDLE_VECTOR: char = '\u{e9cd}';
pub const HEART: char = '\u{e9ce}';
pub const HELP: char = '\u{e9cf}';
pub const HIDE_OFF: char = '\u{e9d0}';
pub const HIDE_ON: char = '\u{e9d1}';
pub const HOLDOUT_OFF: char = '\u{e9d2}';
pub const HOLDOUT_ON: char = '\u{e9d3}';
pub const HOME: char = '\u{e9d4}';
pub const HOOK: char = '\u{e9d5}';
pub const IMAGE: char = '\u{e9d6}';
pub const IMAGE_ALPHA: char = '\u{e9d7}';
pub const IMAGE_BACKGROUND: char = '\u{e9d8}';
pub const IMAGE_DATA: char = '\u{e9d9}';
pub const IMAGE_PLANE: char = '\u{e9da}';
pub const IMAGE_REFERENCE: char = '\u{e9db}';
pub const IMAGE_RGB: char = '\u{e9dc}';
pub const IMAGE_RGB_ALPHA: char = '\u{e9dd}';
pub const IMAGE_ZDEPTH: char = '\u{e9de}';
pub const IMGDISPLAY: char = '\u{e9df}';
pub const IMPORT: char = '\u{e9e0}';
pub const INDIRECT_ONLY_OFF: char = '\u{e9e1}';
pub const INDIRECT_ONLY_ON: char = '\u{e9e2}';
pub const INFO: char = '\u{e9e3}';
pub const INTERPOLATE_BEZIER: char = '\u{e9e4}';
pub const INTERPOLATE_CIRCLE: char = '\u{e9e5}';
pub const INTERPOLATE_CONSTANT: char = '\u{e9e6}';
pub const INTERPOLATE_CUBIC: char = '\u{e9e7}';
pub const INTERPOLATE_EXPONENTIAL: char = '\u{e9e8}';
pub const INTERPOLATE_LINEAR: char = '\u{e9e9}';
pub const INTERPOLATE_QUAD: char = '\u{e9ea}';
pub const INTERPOLATE_QUARTIC: char = '\u{e9eb}';
pub const INTERPOLATE_QUINTIC: char = '\u{e9ec}';
pub const INTERPOLATE_SINE: char = '\u{e9ed}';
pub const INVERSE_SQUARE_CURVE: char = '\u{e9ee}';
pub const IPO_BACK: char = '\u{e9ef}';
pub const IPO_BOUNCE: char = '\u{e9f0}';
pub const IPO_EASE_IN: char = '\u{e9f1}';
pub const IPO_EASE_IN_OUT: char = '\u{e9f2}';
pub const IPO_EASE_OUT: char = '\u{e9f3}';
pub const IPO_ELASTIC: char = '\u{e9f4}';
pub const ITALIC: char = '\u{e9f5}';
pub const KEY_DEHLT: char = '\u{e9f6}';
pub const KEYFRAME: char = '\u{e9f7}';
pub const KEYFRAME_HLT: char = '\u{e9f8}';
pub const KEY_HLT: char = '\u{e9f9}';
pub const KEYINGSET: char = '\u{e9fa}';
pub const LATTICE_DATA: char = '\u{e9fb}';
pub const LAYER_ACTIVE: char = '\u{e9fc}';
pub const LAYER_USED: char = '\u{e9fd}';
pub const LIBRARY_DATA_BROKEN: char = '\u{e9fe}';
pub const LIBRARY_DATA_DIRECT: char = '\u{e9ff}';
pub const LIBRARY_DATA_INDIRECT: char = '\u{ea00}';
pub const LIBRARY_DATA_OVERRIDE: char = '\u{ea01}';
pub const LIGHT: char = '\u{ea02}';
pub const LIGHT_AREA: char = '\u{ea03}';
pub const LIGHT_DATA: char = '\u{ea04}';
pub const LIGHT_HEMI: char = '\u{ea05}';
pub const LIGHT_POINT: char = '\u{ea06}';
pub const LIGHTPROBE_CUBEMAP: char = '\u{ea07}';
pub const LIGHTPROBE_GRID: char = '\u{ea08}';
pub const LIGHTPROBE_PLANAR: char = '\u{ea09}';
pub const LIGHT_SPOT: char = '\u{ea0a}';
pub const LIGHT_SUN: char = '\u{ea0b}';
pub const LINEAR_CURVE: char = '\u{ea0c}';
pub const LINE_DATA: char = '\u{ea0d}';
pub const LINENUMBERS_OFF: char = '\u{ea0e}';
pub const LINENUMBERS_ON: char = '\u{ea0f}';
pub const LINK_BLEND: char = '\u{ea10}';
pub const LINKED: char = '\u{ea11}';
pub const LOCKED: char = '\u{ea12}';
pub const LOCK_VIEW_OFF: char = '\u{ea13}';
pub const LOCK_VIEW_ON: char = '\u{ea14}';
pub const LONGDISPLAY: char = '\u{ea15}';
pub const LOOP_BACK: char = '\u{ea16}';
pub const LOOP_FORWARDS: char = '\u{ea17}';
pub const MARKER: char = '\u{ea18}';
pub const MARKER_HLT: char = '\u{ea19}';
pub const MAT_CLOTH: char = '\u{ea1a}';
pub const MAT_CUBE: char = '\u{ea1b}';
pub const MATERIAL: char = '\u{ea1c}';
pub const MATERIAL_DATA: char = '\u{ea1d}';
pub const MAT_FLUID: char = '\u{ea1e}';
pub const MAT_PLANE: char = '\u{ea1f}';
pub const MAT_SHADERBALL: char = '\u{ea20}';
pub const MAT_SPHERE: char = '\u{ea21}';
pub const MAT_SPHERE_SKY: char = '\u{ea22}';
pub const MEMORY: char = '\u{ea23}';
pub const MENU_PANEL: char = '\u{ea24}';
pub const MESH_CAPSULE: char = '\u{ea25}';
pub const MESH_CIRCLE: char = '\u{ea26}';
pub const MESH_CONE: char = '\u{ea27}';
pub const MESH_CUBE: char = '\u{ea28}';
pub const MESH_CYLINDER: char = '\u{ea29}';
pub const MESH_DATA: char = '\u{ea2a}';
pub const MESH_GRID: char = '\u{ea2b}';
pub const MESH_ICOSPHERE: char = '\u{ea2c}';
pub const MESH_MONKEY: char = '\u{ea2d}';
pub const MESH_PLANE: char = '\u{ea2e}';
pub const MESH_TORUS: char = '\u{ea2f}';
pub const MESH_UVSPHERE: char = '\u{ea30}';
pub const META_BALL: char = '\u{ea31}';
pub const META_CAPSULE: char = '\u{ea32}';
pub const META_CUBE: char = '\u{ea33}';
pub const META_DATA: char = '\u{ea34}';
pub const META_ELLIPSOID: char = '\u{ea35}';
pub const META_PLANE: char = '\u{ea36}';
pub const MOD_ARMATURE: char = '\u{ea37}';
pub const MOD_ARRAY: char = '\u{ea38}';
pub const MOD_BEVEL: char = '\u{ea39}';
pub const MOD_BOOLEAN: char = '\u{ea3a}';
pub const MOD_BUILD: char = '\u{ea3b}';
pub const MOD_CAST: char = '\u{ea3c}';
pub const MOD_CLOTH: char = '\u{ea3d}';
pub const MOD_CURVE: char = '\u{ea3e}';
pub const MOD_DATA_TRANSFER: char = '\u{ea3f}';
pub const MOD_DECIM: char = '\u{ea40}';
pub const MOD_DISPLACE: char = '\u{ea41}';
pub const MOD_DYNAMICPAINT: char = '\u{ea42}';
pub const MOD_EDGESPLIT: char = '\u{ea43}';
pub const MOD_EXPLODE: char = '\u{ea44}';
pub const MOD_FLUID: char = '\u{ea45}';
pub const MOD_FLUIDSIM: char = '\u{ea46}';
pub const MOD_HUE_SATURATION: char = '\u{ea47}';
pub const MODIFIER: char = '\u{ea48}';
pub const MODIFIER_DATA: char = '\u{ea49}';
pub const MODIFIER_OFF: char = '\u{ea4a}';
pub const MODIFIER_ON: char = '\u{ea4b}';
pub const MOD_INSTANCE: char = '\u{ea4c}';
pub const MOD_LATTICE: char = '\u{ea4d}';
pub const MOD_MASK: char = '\u{ea4e}';
pub const MOD_MESHDEFORM: char = '\u{ea4f}';
pub const MOD_MIRROR: char = '\u{ea50}';
pub const MOD_MULTIRES: char = '\u{ea51}';
pub const MOD_NOISE: char = '\u{ea52}';
pub const MOD_NORMALEDIT: char = '\u{ea53}';
pub const MOD_OCEAN: char = '\u{ea54}';
pub const MOD_OFFSET: char = '\u{ea55}';
pub const MOD_OPACITY: char = '\u{ea56}';
pub const MOD_PARTICLE_INSTANCE: char = '\u{ea57}';
pub const MOD_PARTICLES: char = '\u{ea58}';
pub const MOD_PHYSICS: char = '\u{ea59}';
pub const MOD_REMESH: char = '\u{ea5a}';
pub const MOD_SCREW: char = '\u{ea5b}';
pub const MOD_SHRINKWRAP: char = '\u{ea5c}';
pub const MOD_SIMPLEDEFORM: char = '\u{ea5d}';
pub const MOD_SIMPLIFY: char = '\u{ea5e}';
pub const MOD_SKIN: char = '\u{ea5f}';
pub const MOD_SMOOTH: char = '\u{ea60}';
pub const MOD_SOFT: char = '\u{ea61}';
pub const MOD_SOLIDIFY: char = '\u{ea62}';
pub const MOD_SUBSURF: char = '\u{ea63}';
pub const MOD_THICKNESS: char = '\u{ea64}';
pub const MOD_TIME: char = '\u{ea65}';
pub const MOD_TINT: char = '\u{ea66}';
pub const MOD_TRIANGULATE: char = '\u{ea67}';
pub const MOD_UVPROJECT: char = '\u{ea68}';
pub const MOD_VERTEX_WEIGHT: char = '\u{ea69}';
pub const MOD_WARP: char = '\u{ea6a}';
pub const MOD_WAVE: char = '\u{ea6b}';
pub const MOD_WIREFRAME: char = '\u{ea6c}';
pub const MONKEY: char = '\u{ea6d}';
pub const MOUSE_LMB: char = '\u{ea6e}';
pub const MOUSE_LMB_DRAG: char = '\u{ea6f}';
pub const MOUSE_MMB: char = '\u{ea70}';
pub const MOUSE_MMB_DRAG: char = '\u{ea71}';
pub const MOUSE_MOVE: char = '\u{ea72}';
pub const MOUSE_RMB: char = '\u{ea73}';
pub const MOUSE_RMB_DRAG: char = '\u{ea74}';
pub const MUTE_IPO_OFF: char = '\u{ea75}';
pub const MUTE_IPO_ON: char = '\u{ea76}';
pub const NETWORK_DRIVE: char = '\u{ea77}';
pub const NEW_FOLDER: char = '\u{ea78}';
pub const NEXT_KEYFRAME: char = '\u{ea79}';
pub const NLA: char = '\u{ea7a}';
pub const NLA_PUSHDOWN: char = '\u{ea7b}';
pub const NO_CURVE: char = '\u{ea7c}';
pub const NODE: char = '\u{ea7d}';
pub const NODE_COMPOSITING: char = '\u{ea7e}';
pub const NODE_CORNER: char = '\u{ea7f}';
pub const NODE_INSERT_OFF: char = '\u{ea80}';
pub const NODE_INSERT_ON: char = '\u{ea81}';
pub const NODE_MATERIAL: char = '\u{ea82}';
pub const NODE_SEL: char = '\u{ea83}';
pub const NODE_SIDE: char = '\u{ea84}';
pub const NODE_TEXTURE: char = '\u{ea85}';
pub const NODE_TOP: char = '\u{ea86}';
pub const NODETREE: char = '\u{ea87}';
pub const NORMALIZE_FCURVES: char = '\u{ea88}';
pub const NORMALS_FACE: char = '\u{ea89}';
pub const NORMALS_VERTEX: char = '\u{ea8a}';
pub const NORMALS_VERTEX_FACE: char = '\u{ea8b}';
pub const OBJECT_DATA: char = '\u{ea8c}';
pub const OBJECT_DATAMODE: char = '\u{ea8d}';
pub const OBJECT_HIDDEN: char = '\u{ea8e}';
pub const OBJECT_ORIGIN: char = '\u{ea8f}';
pub const ONIONSKIN_OFF: char = '\u{ea90}';
pub const ONIONSKIN_ON: char = '\u{ea91}';
pub const OPTIONS: char = '\u{ea92}';
pub const ORIENTATION_CURSOR: char = '\u{ea93}';
pub const ORIENTATION_GIMBAL: char = '\u{ea94}';
pub const ORIENTATION_GLOBAL: char = '\u{ea95}';
pub const ORIENTATION_LOCAL: char = '\u{ea96}';
pub const ORIENTATION_NORMAL: char = '\u{ea97}';
pub const ORIENTATION_VIEW: char = '\u{ea98}';
pub const ORPHAN_DATA: char = '\u{ea99}';
pub const OUTLINER: char = '\u{ea9a}';
pub const OUTLINER_COLLECTION: char = '\u{ea9b}';
pub const OUTLINER_DATA_ARMATURE: char = '\u{ea9c}';
pub const OUTLINER_DATA_CAMERA: char = '\u{ea9d}';
pub const OUTLINER_DATA_CURVE: char = '\u{ea9e}';
pub const OUTLINER_DATA_EMPTY: char = '\u{ea9f}';
pub const OUTLINER_DATA_FONT: char = '\u{eaa0}';
pub const OUTLINER_DATA_GP_LAYER: char = '\u{eaa1}';
pub const OUTLINER_DATA_GREASEPENCIL: char = '\u{eaa2}';
pub const OUTLINER_DATA_HAIR: char = '\u{eaa3}';
pub const OUTLINER_DATA_LATTICE: char = '\u{eaa4}';
pub const OUTLINER_DATA_LIGHT: char = '\u{eaa5}';
pub const OUTLINER_DATA_LIGHTPROBE: char = '\u{eaa6}';
pub const OUTLINER_DATA_MESH: char = '\u{eaa7}';
pub const OUTLINER_DATA_META: char = '\u{eaa8}';
pub const OUTLINER_DATA_POINTCLOUD: char = '\u{eaa9}';
pub const OUTLINER_DATA_SPEAKER: char = '\u{eaaa}';
pub const OUTLINER_DATA_SURFACE: char = '\u{eaab}';
pub const OUTLINER_DATA_VOLUME: char = '\u{eaac}';
pub const OUTLINER_OB_ARMATURE: char = '\u{eaad}';
pub const OUTLINER_OB_CAMERA: char = '\u{eaae}';
pub const OUTLINER_OB_CURVE: char = '\u{eaaf}';
pub const OUTLINER_OB_EMPTY: char = '\u{eab0}';
pub const OUTLINER_OB_FONT: char = '\u{eab1}';
pub const OUTLINER_OB_FORCE_FIELD: char = '\u{eab2}';
pub const OUTLINER_OB_GREASEPENCIL: char = '\u{eab3}';
pub const OUTLINER_OB_GROUP_INSTANCE: char = '\u{eab4}';
pub const OUTLINER_OB_HAIR: char = '\u{eab5}';
pub const OUTLINER_OB_IMAGE: char = '\u{eab6}';
pub const OUTLINER_OB_LATTICE: char = '\u{eab7}';
pub const OUTLINER_OB_LIGHT: char = '\u{eab8}';
pub const OUTLINER_OB_LIGHTPROBE: char = '\u{eab9}';
pub const OUTLINER_OB_MESH: char = '\u{eaba}';
pub const OUTLINER_OB_META: char = '\u{eabb}';
pub const OUTLINER_OB_POINTCLOUD: char = '\u{eabc}';
pub const OUTLINER_OB_SPEAKER: char = '\u{eabd}';
pub const OUTLINER_OB_SURFACE: char = '\u{eabe}';
pub const OUTLINER_OB_VOLUME: char = '\u{eabf}';
pub const OUTPUT: char = '\u{eac0}';
pub const OVERLAY: char = '\u{eac1}';
pub const PACKAGE: char = '\u{eac2}';
pub const PANEL_CLOSE: char = '\u{eac3}';
pub const PARTICLE_DATA: char = '\u{eac4}';
pub const PARTICLEMODE: char = '\u{eac5}';
pub const PARTICLE_PATH: char = '\u{eac6}';
pub const PARTICLE_POINT: char = '\u{eac7}';
pub const PARTICLES: char = '\u{eac8}';
pub const PARTICLE_TIP: char = '\u{eac9}';
pub const PASTE_DOWN: char = '\u{eaca}';
pub const PASTE_FLIP_DOWN: char = '\u{eacb}';
pub const PASTE_FLIP_UP: char = '\u{eacc}';
pub const PAUSE: char = '\u{eacd}';
pub const PHYSICS: char = '\u{eace}';
pub const PINNED: char = '\u{eacf}';
pub const PIVOT_ACTIVE: char = '\u{ead0}';
pub const PIVOT_CURSOR: char = '\u{ead1}';
pub const PIVOT_INDIVIDUAL: char = '\u{ead2}';
pub const PIVOT_MEDIAN: char = '\u{ead3}';
pub const PIVOT_OBOUNDING_BOX: char = '\u{ead4}';
pub const PLAY: char = '\u{ead5}';
pub const PLAY_REVERSE: char = '\u{ead6}';
pub const PLAY_SOUND: char = '\u{ead7}';
pub const PLUGIN: char = '\u{ead8}';
pub const PLUS: char = '\u{ead9}';
pub const PMARKER: char = '\u{eada}';
pub const PMARKER_ACT: char = '\u{eadb}';
pub const PMARKER_SEL: char = '\u{eadc}';
pub const POINTCLOUD_DATA: char = '\u{eadd}';
pub const POSE_HLT: char = '\u{eade}';
pub const PREFERENCES: char = '\u{eadf}';
pub const PRESET: char = '\u{eae0}';
pub const PRESET_NEW: char = '\u{eae1}';
pub const PREVIEW_RANGE: char = '\u{eae2}';
pub const PREV_KEYFRAME: char = '\u{eae3}';
pub const PROPERTIES: char = '\u{eae4}';
pub const PROPORTIONAL_CENTER_ON: char = '\u{eae5}';
pub const PROPORTIONAL_OFF: char = '\u{eae6}';
pub const PROPORTIONAL_ON: char = '\u{eae7}';
pub const PROPORTIONAL_PROJECTED: char = '\u{eae8}';
pub const QUESTION: char = '\u{eae9}';
pub const QUIT: char = '\u{eaea}';
pub const RADIOBUT_OFF: char = '\u{eaeb}';
pub const RADIOBUT_ON: char = '\u{eaec}';
pub const REC: char = '\u{eaed}';
pub const RECOVER_LAST: char = '\u{eaee}';
pub const REMOVE: char = '\u{eaef}';
pub const RENDER_ANIMATION: char = '\u{eaf0}';
pub const RENDERLAYERS: char = '\u{eaf1}';
pub const RENDER_RESULT: char = '\u{eaf2}';
pub const RENDER_STILL: char = '\u{eaf3}';
pub const RESTRICT_COLOR_OFF: char = '\u{eaf4}';
pub const RESTRICT_COLOR_ON: char = '\u{eaf5}';
pub const RESTRICT_INSTANCED_OFF: char = '\u{eaf6}';
pub const RESTRICT_INSTANCED_ON: char = '\u{eaf7}';
pub const RESTRICT_RENDER_OFF: char = '\u{eaf8}';
pub const RESTRICT_RENDER_ON: char = '\u{eaf9}';
pub const RESTRICT_SELECT_OFF: char = '\u{eafa}';
pub const RESTRICT_SELECT_ON: char = '\u{eafb}';
pub const RESTRICT_VIEW_OFF: char = '\u{eafc}';
pub const RESTRICT_VIEW_ON: char = '\u{eafd}';
pub const REW: char = '\u{eafe}';
pub const RIGHTARROW: char = '\u{eaff}';
pub const RIGHTARROW_THIN: char = '\u{eb00}';
pub const RIGID_BODY: char = '\u{eb01}';
pub const RIGID_BODY_CONSTRAINT: char = '\u{eb02}';
pub const RNA: char = '\u{eb03}';
pub const RNA_ADD: char = '\u{eb04}';
pub const ROOT_CURVE: char = '\u{eb05}';
pub const ROUND_CURVE: char = '\u{eb06}';
pub const SCENE: char = '\u{eb07}';
pub const SCENE_DATA: char = '\u{eb08}';
pub const SCREEN_BACK: char = '\u{eb09}';
pub const SCRIPT: char = '\u{eb0a}';
pub const SCRIPT_PLUGINS: char = '\u{eb0b}';
pub const SCULPTMODE_HLT: char = '\u{eb0c}';
pub const SELECT_DIFFERENCE: char = '\u{eb0d}';
pub const SELECT_EXTEND: char = '\u{eb0e}';
pub const SELECT_INTERSECT: char = '\u{eb0f}';
pub const SELECT_SET: char = '\u{eb10}';
pub const SELECT_SUBTRACT: char = '\u{eb11}';
pub const SEQ_CHROMA_SCOPE: char = '\u{eb12}';
pub const SEQ_HISTOGRAM: char = '\u{eb13}';
pub const SEQ_LUMA_WAVEFORM: char = '\u{eb14}';
pub const SEQ_PREVIEW: char = '\u{eb15}';
pub const SEQ_SEQUENCER: char = '\u{eb16}';
pub const SEQ_SPLITVIEW: char = '\u{eb17}';
pub const SEQ_STRIP_DUPLICATE: char = '\u{eb18}';
pub const SEQ_STRIP_META: char = '\u{eb19}';
pub const SEQUENCE: char = '\u{eb1a}';
pub const SETTINGS: char = '\u{eb1b}';
pub const SHADERFX: char = '\u{eb1c}';
pub const SHADING_BOUNDING_BOX: char = '\u{eb1d}';
pub const SHADING_RENDERED: char = '\u{eb1e}';
pub const SHADING_SOLID: char = '\u{eb1f}';
pub const SHADING_TEXTURE: char = '\u{eb20}';
pub const SHADING_WIRE: char = '\u{eb21}';
pub const SHAPEKEY_DATA: char = '\u{eb22}';
pub const SHARP_CURVE: char = '\u{eb23}';
pub const SHORTDISPLAY: char = '\u{eb24}';
pub const SMALL_CAPS: char = '\u{eb25}';
pub const SMOOTH_CURVE: char = '\u{eb26}';
pub const SNAP_EDGE: char = '\u{eb27}';
pub const SNAP_FACE: char = '\u{eb28}';
pub const SNAP_FACE_CENTER: char = '\u{eb29}';
pub const SNAP_GRID: char = '\u{eb2a}';
pub const SNAP_INCREMENT: char = '\u{eb2b}';
pub const SNAP_MIDPOINT: char = '\u{eb2c}';
pub const SNAP_NORMAL: char = '\u{eb2d}';
pub const SNAP_OFF: char = '\u{eb2e}';
pub const SNAP_ON: char = '\u{eb2f}';
pub const SNAP_PEEL_OBJECT: char = '\u{eb30}';
pub const SNAP_PERPENDICULAR: char = '\u{eb31}';
pub const SNAP_VERTEX: char = '\u{eb32}';
pub const SNAP_VOLUME: char = '\u{eb33}';
pub const SOLO_OFF: char = '\u{eb34}';
pub const SOLO_ON: char = '\u{eb35}';
pub const SORT_ALPHA: char = '\u{eb36}';
pub const SORT_ASC: char = '\u{eb37}';
pub const SORT_BY_EXT: char = '\u{eb38}';
pub const SORT_DESC: char = '\u{eb39}';
pub const SORT_SIZE: char = '\u{eb3a}';
pub const SORT_TIME: char = '\u{eb3b}';
pub const SOUND: char = '\u{eb3c}';
pub const SPEAKER: char = '\u{eb3d}';
pub const SPHERE: char = '\u{eb3e}';
pub const SPHERE_CURVE: char = '\u{eb3f}';
pub const SPREADSHEET: char = '\u{eb40}';
pub const STATUSBAR: char = '\u{eb41}';
pub const STICKY_UVS_DISABLE: char = '\u{eb42}';
pub const STICKY_UVS_LOC: char = '\u{eb43}';
pub const STICKY_UVS_VERT: char = '\u{eb44}';
pub const STRANDS: char = '\u{eb45}';
pub const STROKE: char = '\u{eb46}';
pub const STYLUS_PRESSURE: char = '\u{eb47}';
pub const SURFACE_DATA: char = '\u{eb48}';
pub const SURFACE_NCIRCLE: char = '\u{eb49}';
pub const SURFACE_NCURVE: char = '\u{eb4a}';
pub const SURFACE_NCYLINDER: char = '\u{eb4b}';
pub const SURFACE_NSPHERE: char = '\u{eb4c}';
pub const SURFACE_NSURFACE: char = '\u{eb4d}';
pub const SURFACE_NTORUS: char = '\u{eb4e}';
pub const SYNTAX_OFF: char = '\u{eb4f}';
pub const SYNTAX_ON: char = '\u{eb50}';
pub const SYSTEM: char = '\u{eb51}';
pub const TEMP: char = '\u{eb52}';
pub const TEXT: char = '\u{eb53}';
pub const TEXTURE: char = '\u{eb54}';
pub const TEXTURE_DATA: char = '\u{eb55}';
pub const THREE_DOTS: char = '\u{eb56}';
pub const TIME: char = '\u{eb57}';
pub const TOOL_SETTINGS: char = '\u{eb58}';
pub const TOPBAR: char = '\u{eb59}';
pub const TPAINT_HLT: char = '\u{eb5a}';
pub const TRACKER: char = '\u{eb5b}';
pub const TRACKER_DATA: char = '\u{eb5c}';
pub const TRACKING: char = '\u{eb5d}';
pub const TRACKING_BACKWARDS: char = '\u{eb5e}';
pub const TRACKING_BACKWARDS_SINGLE: char = '\u{eb5f}';
pub const TRACKING_CLEAR_BACKWARDS: char = '\u{eb60}';
pub const TRACKING_CLEAR_FORWARDS: char = '\u{eb61}';
pub const TRACKING_FORWARDS: char = '\u{eb62}';
pub const TRACKING_FORWARDS_SINGLE: char = '\u{eb63}';
pub const TRACKING_REFINE_BACKWARDS: char = '\u{eb64}';
pub const TRACKING_REFINE_FORWARDS: char = '\u{eb65}';
pub const TRANSFORM_ORIGINS: char = '\u{eb66}';
pub const TRASH: char = '\u{eb67}';
pub const TRIA_DOWN: char = '\u{eb68}';
pub const TRIA_DOWN_BAR: char = '\u{eb69}';
pub const TRIA_LEFT: char = '\u{eb6a}';
pub const TRIA_LEFT_BAR: char = '\u{eb6b}';
pub const TRIA_RIGHT: char = '\u{eb6c}';
pub const TRIA_RIGHT_BAR: char = '\u{eb6d}';
pub const TRIA_UP: char = '\u{eb6e}';
pub const TRIA_UP_BAR: char = '\u{eb6f}';
pub const UGLYPACKAGE: char = '\u{eb70}';
pub const UNDERLINE: char = '\u{eb71}';
pub const UNLINKED: char = '\u{eb72}';
pub const UNLOCKED: char = '\u{eb73}';
pub const UNPINNED: char = '\u{eb74}';
pub const URL: char = '\u{eb75}';
pub const USER: char = '\u{eb76}';
pub const UV: char = '\u{eb77}';
pub const UV_DATA: char = '\u{eb78}';
pub const UV_EDGE_SELECT: char = '\u{eb79}';
pub const UV_FACE_SELECT: char = '\u{eb7a}';
pub const UV_ISLANDS_SELECT: char = '\u{eb7b}';
pub const UV_SYNC_SELECT: char = '\u{eb7c}';
pub const UV_VERTEX_SELECT: char = '\u{eb7d}';
pub const VERTEX_SELECT: char = '\u{eb7e}';
pub const VIEW3D: char = '\u{eb7f}';
pub const VIEW_CAMERA: char = '\u{eb80}';
pub const VIEW_ORTHO: char = '\u{eb81}';
pub const VIEW_PAN: char = '\u{eb82}';
pub const VIEW_PERSPECTIVE: char = '\u{eb83}';
pub const VIEW_ZOOM: char = '\u{eb84}';
pub const VIEWZOOM: char = '\u{eb85}';
pub const VISUAL_SELECTION_DISABLED: char = '\u{eb86}';
pub const VISUAL_SELECTION_HIDDEN: char = '\u{eb87}';
pub const VISUAL_SELECTION_SELECTABLE: char = '\u{eb88}';
pub const VISUAL_SELECTION_UNSELECTABLE: char = '\u{eb89}';
pub const VOLUME_DATA: char = '\u{eb8a}';
pub const VPAINT_HLT: char = '\u{eb8b}';
pub const WINDOW: char = '\u{eb8c}';
pub const WORDWRAP_OFF: char = '\u{eb8d}';
pub const WORDWRAP_ON: char = '\u{eb8e}';
pub const WORKSPACE: char = '\u{eb8f}';
pub const WORLD: char = '\u{eb90}';
pub const WORLD_DATA: char = '\u{eb91}';
pub const WPAINT_HLT: char = '\u{eb92}';
pub const X: char = '\u{eb93}';
pub const X_RAY: char = '\u{eb94}';
pub const ZOOM_ALL: char = '\u{eb95}';
pub const ZOOM_IN: char = '\u{eb96}';
pub const ZOOM_OUT: char = '\u{eb97}';
pub const ZOOM_PREVIOUS: char = '\u{eb98}';
pub const ZOOM_SELECTED: char = '\u{eb99}';