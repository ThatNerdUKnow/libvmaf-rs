/// This struct represents the status of VMAF calculation  
///
/// For every frame pair decoded, a `Decode` variant is emitted to the callback function provided to `Vmaf::get_vmaf_scores()`
/// After all frames are decoded, the `GetScore` variants are emitted to `Vmaf::get_vmaf_scores()`
///
/// ### Important!
/// Given that the two [`Video`](../video/struct.Video.html) structs passed to `Vmaf::get_vmaf_scores()` have the same number of frames,
///  the number of times each variant is emitted from `Vmaf::get_vmaf_scores()` is equal to the number of frame pairs.
/// In this way, you may calculate the progress of Vmaf score calculation in this manner:
/// `(# of times a variant has been emitted)/(number of frame pairs)`.
/// One may intuit that the progress of vmaf score calculation occurs in two stages,
/// Decoding, and Retrieving the score. Ideally this should be represented in two seperate progress bars
#[derive(Debug)]
pub enum VmafStatus {
    /// update on the decoding of a video framepair.
    /// Every time a frame pair is decoded and processed, this variant is emitted
    /// to the callback function provided to `Vmaf::get_vmaf_scores()`
    Decode,
    /// this variant is an update on the retrieval of a Vmaf Score after all
    /// frames are decoded and processed.
    /// After all frames are decoded, this variant is emitted to the callback function provided to
    ///`Vmaf::get_vmaf_scores()`
    GetScore,
}