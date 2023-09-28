#![cfg(target_arch = "wasm32")]

mod utils;

use bevy_ecs::{
    system::{Commands, NonSend, Res, SystemState},
    world::World,
};
use common::prelude::*;
use source::prelude::*;
use std::{
    cell::OnceCell,
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};
use std::{
    io::{BufReader, Cursor},
    panic,
    path::PathBuf,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use winit::platform::web::WindowBuilderExtWebSys;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use glam::{Mat4, Vec3, Vec4};

#[wasm_bindgen(start)]
fn run() {}

// First up let's take a look of binding `console.log` manually, without the
// help of `web_sys`. Here we're writing the `#[wasm_bindgen]` annotations
// manually ourselves, and the correctness of our program relies on the
// correctness of these annotations!

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

// Next let's define a macro that's like `println!`, only it works for
// `console.log`. Note that `println!` doesn't actually work on the wasm target
// because the standard library currently just eats all output. To get
// `println!`-like behavior in your app you'll likely want a macro like this.

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

pub async fn vinit(
    canvas: Option<web_sys::HtmlCanvasElement>,
) -> (bsp_explorer::state::StateApp, EventLoop<()>) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_canvas(canvas)
        .build(&event_loop)
        .unwrap();

    (bsp_explorer::vinit_state(window).await, event_loop)
}

#[wasm_bindgen]
pub fn greet() {
    console_log!("Hello, bsp-web!");
}

#[wasm_bindgen]
pub async fn open_folder(files: web_sys::FileList) {
    console_log!("OPned filder");

    for f in 0..files.length() {
        console_log!("{}", files.get(f).unwrap().name())
    }
}

#[wasm_bindgen]
pub async fn run_blank(files: web_sys::FileList, canvas: Option<web_sys::HtmlCanvasElement>) {
    console_log!("Running winit:");
    let (state, event_loop) = vinit(canvas).await;

    console_log!("Starting loop:");
    bsp_explorer::vrun(state, event_loop);
}

// #[wasm_bindgen]
// pub async fn test_load_vpk(blob: web_sys::Blob) {
//     console_log!("Hello, bsp-web!");
//     let buff = wasm_bindgen_futures::JsFuture::from(blob.array_buffer()).await;

//     let array = js_sys::Uint8Array::new(&buff.unwrap());
//     let data = array.to_vec();
//     console_log!("Len: {}", data.len());

//     let cursor = Cursor::new(data);
//     let mut buffer = BufReader::new(cursor);

//     console_log!("Loading vpk");
//     let vpk = bsp_explorer::assets::VPKDirectory::read(&mut buffer, PathBuf::new());

//     console_log!("Eerr: {}", vpk.is_err());
//     console_log!("{:?}", vpk.unwrap().files.keys());
// }

struct FileEntry {
    blob: js_sys::Uint8Array,
    data: OnceCell<Vec<u8>>,
}

#[wasm_bindgen]
pub async fn run_mesh(files: web_sys::FileList, canvas: Option<web_sys::HtmlCanvasElement>) {
    console_log!("Running winit:");
    let (mut state, event_loop) = vinit(canvas).await;

    let mut file_dict: HashMap<String, VFile> = Default::default();

    console_log!("Loading files:");
    for i in 0..files.length() {
        let file = files.get(i).unwrap();
        let name = file.name();
        if name.contains("misc") || name.contains("textures") || name.contains("train") {
            console_log!("Starting {}:", name);

            let fut = wasm_bindgen_futures::JsFuture::from(file.array_buffer());
            let data: Vec<u8> = js_sys::Uint8Array::new(&fut.await.unwrap()).to_vec();
            file_dict.insert(name, VFile { data });
        }
    }

    let files = VFileSystem {
        files: Arc::new(file_dict),
    };

    console_log!("Making game:");
    // Construct a `SystemState` struct, passing in a tuple of `SystemParam`
    // as if you were writing an ordinary system.

    let game_data = GameData::load_game(Game::HalfLife2, files.clone());
    let start_map = game_data.inner.starter_map().to_owned();

    let vmt = game_data.inner.load_vmt(&VSplitPath::new(
        "materials/concrete",
        "concretewall071d",
        "vmt",
    ));

    console_log!("VMT Test: {:?}", vmt);

    state.world_mut().insert_resource(files);
    state.world_mut().insert_resource(game_data);

    state.world_mut().spawn(bsp_explorer::state::command_task(
        "Loading game data",
        || {
            bsp_explorer::state::box_cmds(|commands| {
                commands.add(|w: &mut World| {
                    w.send_event(bsp_explorer::state::MapChangeEvent(start_map))
                });
            })
        },
    ));

    console_log!("Starting loop:");
    bsp_explorer::vrun(state, event_loop);
}
