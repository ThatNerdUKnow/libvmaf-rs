use std::ffi::CStr;

use libvmaf_sys::vmaf_version;

pub fn VmafVersion() -> String {
    let version = unsafe { CStr::from_ptr(vmaf_version()) };
    version.to_str().unwrap().to_string()
}

#[cfg(test)]
mod test{
    use super::VmafVersion;

    #[test]
    fn version(){
        let version = VmafVersion();
        println!("{}",version);
    }
}