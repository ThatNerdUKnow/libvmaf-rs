//! libvmaf-rs intends to be an ergonomic wrapper around the raw library bindings for Netflix's `libvmaf` from `libvmaf-sys`.  
//!
//! VMAF is an Emmy-winning perceptual video quality assessment algorithm developed by Netflix. It is a full-reference metric, meaning that it
//! is calculated on pairs of reference/distorted pictures

/// This module defines a wrapper around VmafPicture. This module also translates between FFMPEG's AVFrame struct and VmafPicture
pub mod picture;

/// This is the module you probably want to look at first
pub mod vmaf;

/// Module concerned with loading VMAFModels
//pub mod model;

/// Utility module, get versions of VMAF and FFMPEG here
pub mod utils;

/// This module is concerned with decoding video files into YUV format
#[cfg(feature="ffmpeg")]
pub mod video;

/// FFI Error types
pub mod error;

pub mod model;
