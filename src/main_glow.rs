use eframe::egui;
use egui::mutex::Mutex;
use std::sync::Arc;

// Define the native options for the application window
pub fn run() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(350.0, 380.0)), // Set the initial window size
        multisampling: 1,                                    // Set the level of multisampling
        renderer: eframe::Renderer::Glow,                    // Use the Glow renderer
        ..Default::default()                                 // Use default options for the rest
    };
    eframe::run_native(
        "Custom 3D painting in eframe using glow", // Set the window title
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))), // Create a new instance of the application
    )
}

// Define the application struct
struct MyApp {
    rotating_triangle: Arc<Mutex<RotatingTriangle>>, // A shared instance of `RotatingTriangle`
    angle: f32,                                      // The current angle of the rotating triangle
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
                ui.hyperlink_to("glow", "https://github.com/grovesNL/glow");
                ui.label(" (OpenGL).");
            });

            // Show the rotating triangle
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });

            // Show a label with instructions for rotating the triangle
            ui.label("Drag to rotate!");
        });
    }

    // Clean up resources when the application exits
    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.rotating_triangle.lock().destroy(gl);
        }
    }
}

// Implement additional methods for `MyApp`
impl MyApp {
    // Create a new instance of `MyApp`
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with the glow backend");
        Self {
            rotating_triangle: Arc::new(Mutex::new(RotatingTriangle::new(gl))),
            angle: 0.0,
        }
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
        let rotating_triangle = self.rotating_triangle.clone();

        // Create a new paint callback that calls `RotatingTriangle::paint`
        let callback_fn = egui_glow::CallbackFn::new(move |_info, painter| {
            rotating_triangle.lock().paint(painter.gl(), angle);
        });
        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(callback_fn),
        };
        ui.painter().add(callback);
    }
}

struct RotatingTriangle {
    program: glow::Program,
    vertex_array: glow::VertexArray,
}

impl RotatingTriangle {
    fn new(gl: &glow::Context) -> Self {
        use glow::HasContext as _;

        let shader_version = if cfg!(target_arch = "wasm32") {
            "#version 300 es"
        } else {
            "#version 330"
        };

        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let (vertex_shader_source, fragment_shader_source) = (
                include_str!("./shaders/glow_compatible/custom3d_shader.vert"),
                include_str!("./shaders/glow_compatible/custom3d_shader.frag"),
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );
                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            Self {
                program,
                vertex_array,
            }
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }

    fn paint(&self, gl: &glow::Context, angle: f32) {
        use glow::HasContext as _;
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_1_f32(
                gl.get_uniform_location(self.program, "u_angle").as_ref(),
                angle,
            );
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}
