use slsengine;
use slsengine_backend_vk as bvk;
use slsengine_platform_sdl as plt;

fn main() {
    let platform = plt::platform().build_vk().expect();
}
