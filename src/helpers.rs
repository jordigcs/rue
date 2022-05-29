pub mod colors {
    #[derive(Copy, Clone)]
    pub struct Color {
        pub r: f64,
        pub g: f64,
        pub b: f64,
        pub a: f64,
    }

    impl Color {
        pub fn rgb(r: f64, g: f64, b: f64) -> Color {
            Color { r, g, b, a: 1.0 }
        }

        pub fn rgba(r: f64, g: f64, b: f64, a: f64) -> Color {
            Color { r, g, b, a }
        }

        pub fn rgb_255(r: u8, g:u8, b:u8) -> Color {
            Color { r: (r as f64) / 255.0, g: (g as f64) / 255.0, b: (b as f64) / 255.0, a: 1.0 }
        }
    
        pub fn rgba_255(r: u8, g:u8, b:u8, a:f64) -> Color {
            Color { r: (r as f64) / 255.0, g: (g as f64) / 255.0, b: (b as f64) / 255.0, a }
        }
    }

    impl Default for Color {
        fn default() -> Self {
            Color::rgb(1.0, 1.0, 1.0)
        }
    }

    pub fn color_to_wgpu_color(color: Color) -> wgpu::Color {
        wgpu::Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}
