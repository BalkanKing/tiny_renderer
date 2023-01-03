use std::time;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use show_image::{WindowOptions, ImageView, ImageInfo, event, create_window};
use obj::raw::{parse_obj};
use nalgebra as na;
use na::vector;

use crate::scene::Scene;
use crate::shader::{ShaderPipeline, DefaultSP, GouraudSP};

// @TODO redo asset_path to be an actual Path object somehow
pub struct Params {
    pub width: u32,
    pub height: u32,
    pub print_fps: bool,
    pub asset_path: String,
}

/// Helper, defining exit event to be an Escape key press.
fn is_exit_event(window_event: event::WindowEvent) -> bool {
    if let event::WindowEvent::KeyboardInput(event) = window_event {
        // println!("{:#?}", event);
        if event.input.key_code == Some(event::VirtualKeyCode::Escape) && event.input.state.is_released() {
            return true;
        }
    }

    return false;
}

/// Actualy launches the window, showing images.
/// Takes struct, defining execution params.
pub fn run(params: Params) -> Result<(), Box<dyn std::error::Error>>{    
    let obj_path = params.asset_path.clone() + "/model.obj";
    let texture_path = params.asset_path.clone() + "/texture.tga";
    let normal_map_path = params.asset_path.clone() + "/normal_map.tga";
    let spec_map_path = params.asset_path.clone() + "/spec_map.tga";

    println!("Loading model from: {}", obj_path);
    let obj = parse_obj(BufReader::new(File::open(obj_path)?))?;
    println!("Number of vertices in a model: {}", obj.positions.len());
    println!("Number of polygons in a model: {}", obj.polygons.len());

    println!("Loading texture from: {}", texture_path);
    let texture = image::open(texture_path)?.into_rgb8();
    println!("Dimensions of loaded texture are: {} x {}", texture.width(), texture.height());

    println!("Loading texture from: {}", normal_map_path);
    let normal_map = image::open(normal_map_path)?.into_rgb8();
    println!("Dimensions of loaded texture are: {} x {}", normal_map.width(), normal_map.height());

    println!("Loading texture from: {}", spec_map_path);
    let spec_map = image::open(spec_map_path)?.into_rgb8();
    println!("Dimensions of loaded texture are: {} x {}", spec_map.width(), spec_map.height());

    // let mut scene = Scene::<DefaultSP>::new(params.width, params.height, model, texture);
    let mut scene = Scene::<GouraudSP>::new(params.width, params.height, obj, texture, normal_map, spec_map);

    let window_options: WindowOptions = WindowOptions {
        size: Some([params.width, params.height]),
        ..Default::default()
    };
    let window = create_window("output", window_options)?;
    let event_channel = window.event_channel()?;

    // Stats.
    let mut exit = false;
    let time_begin = time::Instant::now();
    let mut frame_counter_time_begin = time::Instant::now();
    let mut frame_counter: u32 = 0;
    while !exit {
        let passed_time = time::Instant::now()
        .duration_since(time_begin)
        .as_secs_f32();

        // Clearing z-buffer and resetting rendered data to (0, 0, 0).
        scene.clear();        

        // Setting up camera position and direction.
        let look_from = vector![1.0 * passed_time.sin(), 0.0, 1.0 * passed_time.cos()];
        // let look_from = vector![0.0, 0.0, 1.0];
        let look_at = vector![0.0, 0.0, 0.0];
        let up = vector![0.0, 1.0, 0.0];
        // Setting up the light.
        scene.set_light_direction(vector![0.0, 0.0, 1.0].normalize());
        // scene.set_light_direction(vector![1.0 * passed_time.sin(), 0.0, 1.0 * passed_time.cos()].normalize());
        // Preparing transforms, initializing shader buffer.
        scene.prepare_render(look_from, look_at, up);
        // Rendering the current frame.
        scene.render();

        // Getting rendered data as a data slice and feeding it into window.
        let data = scene.get_render_data();
        // let data = scene.get_depth_data();
        let image_view = ImageView::new(ImageInfo::rgb8(params.width, params.height), data.as_raw());
        window.set_image("image", image_view)?;

        // Unloading all the garbage from event channel, that has piled up, looking for exit event.
        let exit_poll_result = event_channel.try_iter()
        .map(|window_event| is_exit_event(window_event))
        .reduce(|was_exit_event, is_exit_event| was_exit_event || is_exit_event);

        // If any event is Escape key press, then exiting.
        exit = match exit_poll_result {
            Some(value) => value,
            None => false,
        };
        
        if params.print_fps {
            // Counting frames to printout stats every seconds.
            frame_counter += 1;
            if time::Instant::now()
            .duration_since(frame_counter_time_begin)
            .as_secs_f32() > 1.0 {
                println!("FPS --- {}", frame_counter);
                frame_counter_time_begin = time::Instant::now();
                frame_counter = 0;
            }
        }
    }

    return Ok(());
}