use state::{Gpu, State};
use wgpu::{
    Backends, CompositeAlphaMode, DeviceDescriptor, Dx12Compiler, Features, Gles3MinorVersion,
    Instance, InstanceDescriptor, InstanceFlags, Limits, PowerPreference, PresentMode,
    RequestAdapterOptions, TextureUsages,
};
use winit::{dpi, event_loop::EventLoopBuilder, window::Window};

use crate::event_handling::run_event_loop;

const SCREEN_WIDTH: u32 = 768;
const SCREEN_HEIGHT: u32 = 768;

mod event_handling;
mod imgui_state;
mod rendering;
mod state;

fn main() {
    env_logger::init();
    let event_loop = EventLoopBuilder::default()
        .build()
        .expect("Couldn't create event loop");

    let window = event_loop.create_window(Window::default_attributes().with_inner_size(dpi::PhysicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT))).expect("Couldn't create window");
    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        flags: InstanceFlags::default(),
        dx12_shader_compiler: Dx12Compiler::Fxc,
        gles_minor_version: Gles3MinorVersion::Automatic,
    });

    let surface = instance
        .create_surface(&window)
        .expect("Unable to create surface");

    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .expect("Unable to request adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(
        &DeviceDescriptor {
            label: None,
            required_features: Features::default() | Features::POLYGON_MODE_LINE,
            required_limits: Limits::downlevel_webgl2_defaults(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .expect("Unable to request device");

    let config = wgpu::SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_capabilities(&adapter).formats[0],
        width: SCREEN_WIDTH,
        height: SCREEN_HEIGHT,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: vec![surface.get_capabilities(&adapter).formats[0]],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    let gpu = Gpu::new(surface, device, queue, config);
    let mut state = State::new(gpu, &window);
    event_loop
        .run(|event, window_target| run_event_loop(event, window_target, &window, &mut state))
        .unwrap()
}
