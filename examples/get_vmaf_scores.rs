use std::{env, rc::Rc};

use indicatif::{ProgressBar, ProgressStyle};
use libvmaf_rs::{
    picture::Picture,
    scoring::{config::ModelConfig, model::Model},
    video::Video,
    vmaf::Vmaf2,
};
use libvmaf_sys::VmafLogLevel;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let reference: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1280, 720).unwrap();
    let distorted: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1280, 720).unwrap();

    let num_frames = reference.len();

    let model = Model::new(ModelConfig::default(), "vmaf_v0.6.1".to_owned())
        .expect("Can't load vmaf model");
    //let model = Model::load_model(ModelConfig::default(), "./examples/vmaf_v0.6.1.json").unwrap();

    let mut vmaf = Vmaf2::new(
        VmafLogLevel::VMAF_LOG_LEVEL_DEBUG,
        num_cpus::get() as u32,
        0,
        0,
        Rc::new(model),
    )
    .unwrap();

    let style =
        ProgressStyle::with_template("{prefix}: {eta_precise} {wide_bar} [{pos}/{len}]").unwrap();

    let decode_progress = ProgressBar::new(num_frames.try_into().unwrap())
        .with_prefix("Calculating Vmaf Scores")
        .with_style(style.clone());

    let get_score_progress = ProgressBar::new(num_frames.try_into().unwrap())
        .with_prefix("Getting scores")
        .with_style(style);

    let framepairs = reference.into_iter().zip(distorted.into_iter());

    for (index, (reference, distorted)) in framepairs.into_iter().enumerate() {
        let reference: Picture = reference
            .try_into()
            .expect(&format!("Couldn't get reference frame at index {index}"));
        let distorted: Picture = distorted
            .try_into()
            .expect("Couldn't get distorted frame at index {index}");

        vmaf.read_framepair(reference, distorted, index as u32)
            .expect("Coudldn't read framepair");
        decode_progress.inc(1);
    }

    vmaf.flush_framebuffers()
        .expect("Couldn't flush frame buffers");

    decode_progress.finish();

    let scores: f64 = (1..num_frames)
        .into_iter()
        .map(|i| {
            let score = vmaf
                .get_score_at_index(i as u32)
                .expect(&format!("Can't get score for frame {i}"));
            get_score_progress.inc(1);
            score
        })
        .sum::<f64>()
        / num_frames as f64;

    get_score_progress.finish();

    println!("Pooled VMAF Score: {scores}");
}
