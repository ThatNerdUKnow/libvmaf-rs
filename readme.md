libvmaf-rs intends to be an ergonomic wrapper around the raw library bindings for Netflix's `libvmaf` from `libvmaf-sys`.  

VMAF is an Emmy-winning perceptual video quality assessment algorithm developed by Netflix. It is a full-reference metric, meaning that it
is calculated on pairs of reference/distorted pictures

## Getting started:

First, construct `Video`s from video files for both your reference and distorted(compressed) video files.  

This example uses the same file for both `reference` and `distorted`, but normally distorted would be a compressed video while reference would point to the original, uncompressed video
```rs
let reference: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1920, 1080).unwrap();
let distorted: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1920, 1080).unwrap();
```

Now, you need to load a model,
```rs
let model: Model = Model::default();
```

Optionally, you may define a callback function. This is useful if you want updates on the progress of VMAF score calculation
```rs
let callback = |status: VmafStatus| match status {
VmafStatus::Decode => dostuff(),
VmafStatus::GetScore => dostuff(),
};
```

Now we construct a `Vmaf` context
```rs
let vmaf = Vmaf::new(
VmafLogLevel::VMAF_LOG_LEVEL_DEBUG,
num_cpus::get().try_into().unwrap(),
0,
0,
)
```

To get a vector of scores for every frame, we may use the following method on our new `Vmaf` context:
```rs
let scores = vmaf
.get_vmaf_scores(reference, distorted, model, Some(callback))
.unwrap();
```