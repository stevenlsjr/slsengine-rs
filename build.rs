use std::env;
use std::path::*;

fn main() {
    use std::process::Command;
    let sdl_path = env::var("SDL_LIB_PATH");

    
    let vk_path = if cfg!(target_os = "macos") {
        env::var("VULKAN_SDK").map(|path_str| {
            let mut path = PathBuf::from(&path_str);
            path.push("lib");
            path.to_str().unwrap().to_owned()
        })
    } else {
        env::var("VK_LIB_PATH")
    };
    if let Ok(path) = sdl_path {
        println!(r"cargo:rustc-link-search={}", path);
    }

    if let Ok(path) = vk_path {
        println!(r"cargo:rustc-link-search={}", path);
    }
}
