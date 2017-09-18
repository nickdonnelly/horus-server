use super::models::{HImage, HVideo, HPaste};

#[derive(Serialize)]
pub struct ImageList {
    pub first_name: String,
    pub images: Vec<HImage>
}

#[derive(Serialize)]
pub struct VideoList {
    pub first_name: String,
    pub videos: Vec<HVideo>,
}

pub struct PasteList {
    pub first_name: String,
    pub pastes: Vec<HPaste>
}

#[derive(Serialize)]
pub struct ManageImage {
    pub id: String,
    pub img_title: String,
}

#[derive(Serialize)]
pub struct ManageVideo {
    pub id: String,
    pub video_title: String,
}

#[derive(Serialize)]
pub struct ManagePaste {
    pub id: String,
    pub paste_title: String,
    pub paste_content: String,
}

