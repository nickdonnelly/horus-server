use super::models::{HImage, HVideo, HPaste, HFile};

#[derive(Serialize)]
pub struct ImageList {
    pub title: String,
    pub page_title: String,
    pub editable: bool,
    pub images: Vec<HImage>
}

#[derive(Serialize)]
pub struct VideoList {
    pub title: String,
    pub page_title: String,
    pub editable: bool,
    pub videos: Vec<HVideo>,
}

#[derive(Serialize)]
pub struct PasteList {
    pub title: String,
    pub page_title: String,
    pub pastes: Vec<HPaste>,
    pub editable: bool,
}

#[derive(Serialize)]
pub struct FileList {
    pub title: String,
    pub page_title: String,
    pub files: Vec<HFile>,
    pub editable: bool,
}

#[derive(Serialize)]
pub struct ManageImage {
    pub id: String,
    pub title: String,
    pub page_title: String,
    pub editable: bool,
}

#[derive(Serialize)]
pub struct ManageVideo {
    pub id: String,
    pub title: String,
    pub page_title: String,
    pub editable: bool,
}

#[derive(Serialize)]
pub struct ManagePaste {
    pub id: String,
    pub title: String,
    pub page_title: String,
    pub paste: HPaste,
    pub editable: bool,
}
