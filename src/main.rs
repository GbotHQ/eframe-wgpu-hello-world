mod main_wgpu;
use main_wgpu::run;

fn main() {
    // Use ? to propagate the error
    run().unwrap_or_else(|err| {
        eprintln!("An error occurred: {}", err);
        std::process::exit(1);
    });
}
