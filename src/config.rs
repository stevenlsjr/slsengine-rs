#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub window_size: (u32, u32),
    pub window_title: String,
    pub antialiasing: AntiAliasing,
    pub allow_highdpi: bool,
    pub fullscreen: bool
}

impl Default for PlatformConfig {
    fn default() -> Self {
        PlatformConfig {
            window_size: (1280, 960),
            window_title: "Rust OpenGL demo".to_owned(),
            antialiasing: AntiAliasing::None,
            allow_highdpi: true,
            fullscreen: false
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum AntiAliasing {
    None,
    MSAAx2,
    MSAAx4,
    MSAAx8,
}

impl AntiAliasing {
    pub fn n_samples(&self) -> usize {
        match self {
            AntiAliasing::None => 0,
            AntiAliasing::MSAAx2 => 2,
            AntiAliasing::MSAAx4 => 4,
            AntiAliasing::MSAAx8 => 8,
        }
    }
}
