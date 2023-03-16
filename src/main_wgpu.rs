use eframe::egui;
use std::{num::NonZeroU64, sync::Arc};
use wgpu::util::DeviceExt;

// Define the options for the application window
pub fn run() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(350.0, 380.0)), // Set the initial window size
        multisampling: 1,                                    // Set the level of multisampling
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Custom 3D painting in eframe using wgpu", // Set the window title
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))), // Create a new instance of the application
    )
}

// Define the application struct
struct MyApp {
    angle: f32, // The current angle of the rotating triangle
}

// Implement the `eframe::App` trait for `MyApp`
impl eframe::App for MyApp {
    // Update the application state and UI every frame
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Show the UI
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show a label with a hyperlink to the Glow repository
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("The triangle is being painted using ");
                ui.hyperlink_to("wgpu", "https://github.com/gfx-rs/wgpu");
                ui.label(" (WebGPU).");
            });

            // Show the rotating triangle
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });

            // Show a label with instructions for rotating the triangle
            ui.label("Drag to rotate!");
        });
    }
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("You need to run eframe with the WGPU backend");
        let device = &render_state.device;

        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("triangle vs"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("./compiled/custom3d_naga_shader_vert.wgsl").into(),
            ),
        });
        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("triangle fs"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("./compiled/custom3d_naga_shader_frag.wgsl").into(),
            ),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind group"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(16),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[Some(render_state.target_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::cast_slice(&[0.0_f32; 4]), // 16 bytes aligned!
            // Mapping at creation (as done by the create_buffer_init utility) doesn't require us to to add the MAP_WRITE usage
            // (this *happens* to workaround this bug )
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(RotatingTriangle {
                pipeline,
                bind_group,
                uniform_buffer,
            });
        Self { angle: 0.0 }
    }

    // Paint the rotating triangle
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        // Allocate space for the triangle and enable dragging
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

        // Update the angle of the rotating triangle based on mouse input
        self.angle += response.drag_delta().x * 0.01;

        // Clone `self` fields so we can move them into the paint callback
        let angle = self.angle;

        // The callback function for WGPU is in two stages: prepare, and paint.
        //
        // The prepare callback is called every frame before paint and is given access to the wgpu
        // Device and Queue, which can be used, for instance, to update buffers and uniforms before
        // rendering.
        //
        // You can use the main `CommandEncoder` that is passed-in, return an arbitrary number
        // of user-defined `CommandBuffer`s, or both.
        // The main command buffer, as well as all user-defined ones, will be submitted together
        // to the GPU in a single call.
        //
        // The paint callback is called after prepare and is given access to the render pass, which
        // can be used to issue draw commands.
        let callback_fn = egui_wgpu::CallbackFn::new()
            .prepare(move |_device, queue, _encoder, paint_callback_resources| {
                let resources: &RotatingTriangle = paint_callback_resources.get().unwrap();
                resources.prepare(queue, angle);
                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let resources: &RotatingTriangle = paint_callback_resources.get().unwrap();
                resources.paint(render_pass);
            });
        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(callback_fn),
        };

        ui.painter().add(callback);
    }
}

struct RotatingTriangle {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
}

impl RotatingTriangle {
    fn prepare(&self, queue: &wgpu::Queue, angle: f32) {
        // Update our uniform buffer with the angle from the UI
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[angle]));
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        // Draw the triangle
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
