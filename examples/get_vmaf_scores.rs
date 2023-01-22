use libvmaf_rs::{
    model::{config::ModelConfig, Model},
    video::Video,
    vmaf::{status::VmafStatus, Vmaf},
};

fn main() {
    let vmaf = Vmaf::default();

    let reference: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1280, 720).unwrap();
    let distorted: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1280, 720).unwrap();

    let model_config = ModelConfig::default();
    let model = Model::new(model_config, String::from("vmaf_v0.6.1")).unwrap();

    let callback = |status: VmafStatus| match status {
        VmafStatus::Decode => (),
        VmafStatus::GetScore => (),
    };

    let scores = vmaf
        .get_vmaf_scores(reference, distorted, model, Some(callback))
        .unwrap();

    for (index,score) in scores.into_iter().enumerate() {
        println!("Score at frame {index}: {score}")
    }
}
