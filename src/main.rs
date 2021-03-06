use winit::{
    event::*,
    event_loop::{self, ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    },
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {} // *control_flow = ControlFlow::Wait;
                // match event {
                //     Event::WindowEvent {
                //         event: WindowEvent::CloseRequested,
                //         window_id,
                //     } if window_id == window.id() => *control_flow = ControlFlow::Exit,
                //     Event::WindowEvent {
                //         event: WindowEvent::KeyboardInput { input, .. },
                //         window_id,
                //     } if window_id == window.id() => {
                //         if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                //             *control_flow = ControlFlow::Exit
                //         }
                //     }
                //     _ => (),
                // }
    });
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    clear_color: wgpu::Color,
}

impl State {
    async fn new(window: &Window) -> State {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Option::Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let features = adapter.features();
        println!("{:?}", features);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            present_mode: wgpu::PresentMode::Fifo,
            height: size.height,
            width: size.width,
        };

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color: wgpu::Color::WHITE,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position: pos, .. } => {
                println!("{:?}", pos);
                println!("{}", pos.x / self.size.width as f64);
                self.clear_color = wgpu::Color {
                    r: pos.x / self.size.width as f64,
                    g: pos.y / self.size.height as f64,
                    b: 0.2,
                    a: 1.0,
                };

                return true;
            }, 
            _ => {}
        }


        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Encoder"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn main() {
    // env_logger::init();
    // let event_loop = EventLoop::new();
    // let window = WindowBuilder::new().build(&event_loop).unwrap();
    //
    //
    // pollster::block_on(State::new(&window));

    pollster::block_on(run());
}
