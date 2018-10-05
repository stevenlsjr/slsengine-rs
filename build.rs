use std::env;

fn main() {
    let sdl_path = env::var("SDL_LIB_PATH");
    let vk_path = env::var("VK_LIB_PATH");
    if let Ok(path) = sdl_path {
        println!(r"cargo:rustc-link-search={}", path);
    }

    if let Ok(path) = vk_path {
        println!(r"cargo:rustc-link-search={}", path);
    }
}
