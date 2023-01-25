use std::ffi::CStr;
pub use ffmpeg_the_third::util::version as ffmpeg_version;


pub fn vmaf_version() -> String {
    let version = unsafe { CStr::from_ptr(libvmaf_sys::vmaf_version()) };
    version.to_str().unwrap().to_string()
}

#[cfg(test)]
mod test{
    use super::vmaf_version;

    #[test]
    fn version(){
        let version = vmaf_version();
        println!("{}",version);
    }
}