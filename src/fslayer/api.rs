use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct FMeta {
    pub name: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub enum FItemType {
    #[default]
    Folder,
    File,
}

impl FItemType {
    pub fn is_folder(&self) -> bool {
        matches!(self, FItemType::Folder)
    }

    pub fn is_file(&self) -> bool {
        matches!(self, FItemType::File)
    }
}

#[derive(Debug, Serialize)]
pub struct OwnedFlatFItem {
    pub name: String,
    pub entry_type: FItemType,
}

#[derive(Debug, Serialize)]
pub struct OwnedFlatFTree {
    pub meta: FMeta,
    pub sub_entries: Vec<OwnedFlatFItem>,
    pub entry_type: FItemType,
}
