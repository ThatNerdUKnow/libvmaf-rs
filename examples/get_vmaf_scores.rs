use indicatif::{ProgressBar, ProgressStyle};
use libvmaf_rs::{
    model::{config::ModelConfig, Model},
    video::Video,
    vmaf::{status::VmafStatus, Vmaf},
};

fn main() {
    let vmaf = Vmaf::default();

    let reference: Video = Video::new("./video/Big Buck Bunny 720P.m4v", 1280, 720).unwrap();
    let distorted: Video = Video::new("./video/Big Buck Bunny 720P.m4v", 1280, 720).unwrap();

    let num_frames = reference.len();

    let model_config = ModelConfig::default();
    let model = Model::new(model_config, String::from("vmaf_v0.6.1")).unwrap();

    let style =
        ProgressStyle::with_template("{prefix}: {eta_precise} {wide_bar} [{pos}/{len}]").unwrap();

    let decode_progress = ProgressBar::new(num_frames.try_into().unwrap())
        .with_prefix("Calculating Vmaf Scores")
        .with_style(style.clone());

    let get_score_progress = ProgressBar::new(num_frames.try_into().unwrap())
        .with_prefix("Getting scores")
        .with_style(style);

    let callback = |status: VmafStatus| match status {
        VmafStatus::Decode => decode_progress.inc(1),
        VmafStatus::GetScore => get_score_progress.inc(1),
    };

    let scores = vmaf
        .get_vmaf_scores(reference, distorted, model, Some(callback))
        .unwrap();

    decode_progress.finish();
    get_score_progress.finish();

    let average: f64 = scores.into_iter().sum::<f64>() / num_frames as f64;

    println!("Pooled VMAF Score: {average}");
}
