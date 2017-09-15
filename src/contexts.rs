use super::models::{HImage, HVideo, HPaste};

#[derive(Serialize)]
pub struct ImageList {
    pub first_name: String,
    pub images: Vec<HImage>
}

pub struct VideoList {
    pub first_name: String,
    pub videos: Vec<HVideo>,
}

pub struct PasteList {
    pub first_name: String,
    pub pastes: Vec<HPaste>
}
