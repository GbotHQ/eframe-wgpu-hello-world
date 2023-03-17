use naga::{back::wgsl, valid::Capabilities};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = fs::read_dir("src/shaders").unwrap();
    for path in paths {
        let path = path.unwrap().path();

        if let Some(ext) = path.extension() {
            if ext == "frag" || ext == "vert" {
                let shader_type = path
                    .extension()
                    .expect("missing extension")
                    .to_str()
                    .unwrap();
                let out_path = PathBuf::from("src/shaders/compiled/")
                    .join(path.file_name().expect("missing file name"));
                let out_path = format!("{}.wgsl", out_path.to_str().unwrap());

                let source = fs::read_to_string(&path)?;

                let kind = match shader_type {
                    "vert" => shaderc::ShaderKind::Vertex,
                    "frag" => shaderc::ShaderKind::Fragment,
                    other => panic!("Unrecognized file extension .{}", other),
                };

                let binary_result = glsl_to_spirv(source.as_str(), kind)?;
                let wgsl = spirv_to_wgsl(&binary_result)?;
                fs::write(out_path, wgsl)?;
            }
        }
    }

    Ok(())
}

fn spirv_to_wgsl(spirv: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    // spirv to wgsl
    // based on https://github.com/gfx-rs/naga/blob/master/cli/src/bin/naga.rs
    let options = naga::front::spv::Options {
        adjust_coordinate_space: false,
        strict_capabilities: false,
        block_ctx_dump_prefix: None,
    };
    let module = naga::front::spv::parse_u8_slice(spirv, &options)?;

    // Decide which capabilities our output formats can support
    let validation_capabilities =
        Capabilities::all() & !Capabilities::CLIP_DISTANCE | Capabilities::CULL_DISTANCE;

    let validation_flags = naga::valid::ValidationFlags::all();

    // validate the IR
    let info = match naga::valid::Validator::new(validation_flags, validation_capabilities)
        .validate(&module)
    {
        Ok(info) => Some(info),
        Err(error) => {
            return Err(format!("Failed to validate SPIR-V module: {}", error).into());
        }
    };

    let wgsl = wgsl::write_string(&module, info.as_ref().unwrap(), wgsl::WriterFlags::empty())?;

    Ok(wgsl)
}

fn glsl_to_spirv(
    source: &str,
    kind: shaderc::ShaderKind,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let compiler = shaderc::Compiler::new().unwrap();

    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_target_env(
        shaderc::TargetEnv::Vulkan,
        shaderc::EnvVersion::Vulkan1_2 as u32,
    );
    options.set_auto_bind_uniforms(true);
    options.set_auto_map_locations(true);
    options.set_forced_version_profile(460u32, shaderc::GlslProfile::Core);
    let binary_result =
        compiler.compile_into_spirv(source, kind, "shader.glsl", "main", Some(&options))?;

    assert_eq!(Some(&0x07230203), binary_result.as_binary().first());
    Ok(binary_result.as_binary_u8().to_vec())
}
