use models::{HFile, FixedDateHImage, HPaste, HVideo};

#[derive(Serialize)]
pub struct ImageList
{
    pub title: String,
    pub page_title: String,
    pub editable: bool,
    pub images: Vec<FixedDateHImage>,
}

#[derive(Serialize)]
pub struct VideoList
{
    pub title: String,
    pub page_title: String,
    pub editable: bool,
    pub videos: Vec<HVideo>,
}

#[derive(Serialize)]
pub struct PasteList
{
    pub title: String,
    pub page_title: String,
    pub pastes: Vec<HPaste>,
    pub editable: bool,
}

#[derive(Serialize)]
pub struct FileList
{
    pub title: String,
    pub page_title: String,
    pub files: Vec<HFile>,
    pub editable: bool,
}

#[derive(Serialize)]
pub struct ManageImage
{
    pub id: String,
    pub title: String,
    pub page_title: String,
    pub editable: bool,
    pub date_added: String,
    pub password: Option<String>,
    pub img_src: Option<String>,
    pub is_expiry: bool
}

#[derive(Serialize)]
pub struct ManageVideo
{
    pub id: String,
    pub title: String,
    pub page_title: String,
    pub editable: bool,
    pub date_added: String,
    pub is_expiry: bool,
    pub password: Option<String>,
    pub vid_src: Option<String>
}

#[derive(Serialize)]
pub struct ManageFile
{
    pub id: String,
    pub filename: String,
    pub page_title: String,
    pub date_added: String,
    pub is_expiry: bool,
    pub password: Option<String>,
    pub editable: bool
}

#[derive(Serialize)]
pub struct ManagePaste
{
    pub id: String,
    pub title: String,
    pub page_title: String,
    pub paste: HPaste,
    pub editable: bool,
}

#[derive(Serialize)]
pub struct ShowPaste
{
    pub item: HPaste,
    pub meta_tag: Option<String>,
}

#[derive(Serialize)]
pub struct ShowVideo
{
    pub item: HVideo,
    pub meta_tag: Option<String>,
    pub password: bool,
}

#[derive(Serialize)]
pub struct ShowImage
{
    pub password: bool,
    pub item: FixedDateHImage,
    pub meta_tag: Option<String>,
}

#[derive(Serialize)]
pub struct ShowAccount
{
    pub user_id: i32,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: String,
    pub privilege_level: String,
    pub resource_count: i64,
}
