//! libvmaf-rs intends to be an ergonomic wrapper around the raw library bindings for Netflix's `libvmaf` from `libvmaf-sys`.  
//!
//! VMAF is an Emmy-winning perceptual video quality assessment algorithm developed by Netflix. It is a full-reference metric, meaning that it
//! is calculated on pairs of reference/distorted pictures
//!
//! ## Getting started:
//!
//! First, construct [`Video`](video/struct.Video.html)s from video files for both your reference and distorted(compressed) video files.  
//!
//! This example uses the same file for both `reference` and `distorted`, but normally distorted would be a compressed video while reference would point to the original, uncompressed video
//! ```
//! let reference: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1920, 1080).unwrap();
//! let distorted: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1920, 1080).unwrap();
//! ```
//!
//! Now, you need to load a [`Model`](model/struct.Model.html),
//! ```
//! let model_config: ModelConfig = ModelConfig::default();
//! let model: Model = Model::new(model_config, "vmaf_v0.6.1".to_string()).unwrap();
//! ```
//!
//! Optionally, you may define a callback function. This is useful if you want updates on the progress of VMAF score calculation.
//! Refer to the VmafStatus Enum for explanation
//! ```
//! let callback = |status: VmafStatus| match status {
//! VmafStatus::Decode => dostuff(),
//! VmafStatus::GetScore => dostuff(),
//! };
//! ```
//!
//! Now we construct a [`Vmaf`](vmaf/struct.Vmaf.html) context
//! ```
//! let vmaf = Vmaf::new(
//! VmafLogLevel::VMAF_LOG_LEVEL_DEBUG,
//! num_cpus::get().try_into().unwrap(),
//! 0,
//! 0,
//! )
//! ```
//!
//! To get a vector of scores for every frame, we may use the following method on our new `Vmaf` context:
//! ```
//! let scores = vmaf
//! .get_vmaf_scores(reference, distorted, model, Some(callback))
//! .unwrap();
//! ```

/// This module defines a wrapper around VmafPicture. This module also translates between FFMPEG's AVFrame struct and VmafPicture
pub mod picture;

/// This is the module you probably want to look at first
pub mod vmaf;

/// Module concerned with loading VMAFModels
pub mod model;

/// Utility module, get versions of VMAF and FFMPEG here
pub mod utils;

/// This module is concerned with decoding video files into YUV format
pub mod video;

/// FFI Error types
pub mod error;
