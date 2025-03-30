use glow::HasContext;
use stb_image::stb_image as image;

pub struct TextureData {
    pub width: i32,
    pub height: i32,
    pub format: u32,
    pub data: &'static [u8],
}

impl TextureData {
    pub fn new(path: &str) -> Self {
        unsafe { image::stbi_set_flip_vertically_on_load(true as _) };

        let (mut width, mut height, mut channels) = (0, 0, 0);
        let data = unsafe {
            let cs = std::ffi::CString::new(path).unwrap();
            image::stbi_load(
                cs.as_ptr(),
                &raw mut width,
                &raw mut height,
                &raw mut channels,
                0,
            )
        };

        assert!(!data.is_null(), "Failed to load image");

        let format = match channels {
            3 => glow::RGB,
            4 => glow::RGBA,
            _ => panic!("Unsupported image format"),
        };

        TextureData {
            width,
            height,
            format,
            data: unsafe { std::slice::from_raw_parts(data, (width * height * channels) as usize) },
        }
    }
}

impl Drop for TextureData {
    fn drop(&mut self) {
        unsafe {
            image::stbi_image_free(std::mem::transmute::<*const u8, *mut std::ffi::c_void>(
                self.data.as_ptr(),
            ))
        };
    }
}

pub fn setup_texutre_params(gl: &glow::Context, target: u32) {
    unsafe {
        gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_S, glow::REPEAT as _);
        gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_T, glow::REPEAT as _);

        gl.tex_parameter_i32(
            target,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST_MIPMAP_LINEAR as _,
        );

        gl.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, glow::NEAREST as _);
    }
}
