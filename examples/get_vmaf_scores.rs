use std::env;

use indicatif::{ProgressBar, ProgressStyle};
use libvmaf_rs::{
    picture::Picture,
    video::Video,
    vmaf::Vmaf2, model::{config::ModelConfig, Model},
};
use libvmaf_sys::VmafLogLevel;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let reference: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1280, 720).unwrap();
    let distorted: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1280, 720).unwrap();

    let num_frames = reference.len();

    /*let mut model = Model::new(ModelConfig::default(), "vmaf_v0.6.1".to_owned())
    .expect("Can't load vmaf model");*/
    let mut model =
        Model::load_model(ModelConfig::default(), "./examples/vmaf_v0.6.1.json").unwrap();

    let mut vmaf = Vmaf2::new(
        VmafLogLevel::VMAF_LOG_LEVEL_DEBUG,
        num_cpus::get() as u32,
        0,
        0,
    )
    .unwrap();

    vmaf.use_features_from_model(&mut model)
        .expect("Can't load model");

    let style =
        ProgressStyle::with_template("{prefix}: {eta_precise} {wide_bar} [{pos}/{len}]").unwrap();

    let decode_progress = ProgressBar::new(num_frames.try_into().unwrap())
        .with_prefix("Calculating Vmaf Scores")
        .with_style(style.clone());

    let get_score_progress = ProgressBar::new(num_frames.try_into().unwrap())
        .with_prefix("Getting scores")
        .with_style(style);

    let framepairs = reference.into_iter().zip(distorted.into_iter());

    let frame_indicies = framepairs
        .enumerate()
        .map(|(i, (reference, distorted))| {
            let i = i + 1;

            let reference: Picture = reference
                .try_into()
                .expect(&format!("Couldn't get reference frame at index {i}"));
            let distorted: Picture = distorted
                .try_into()
                .expect(&format!("Couldn't get distorted frame at index {i}"));
            vmaf.read_framepair(reference, distorted, i as u32)
                .expect(&format!("Couldn't read framepair at index {i}"));
            decode_progress.inc(1);
            i
        })
        .collect::<Vec<_>>();

    vmaf.flush_framebuffers()
        .expect("Couldn't flush frame buffers");

    decode_progress.finish();

    let scores = frame_indicies.iter().map(|i| {
        let score = vmaf
            .get_score_at_index(&mut model, *i as u32)
            .expect("Couldn't get score");
        get_score_progress.inc(1);
        score
    });

    get_score_progress.finish();

    let sum: f64 = scores.sum();
    let mean = sum / f64::from(num_frames as u32);
    println!("Pooled score: {mean}");
}
