use crate::fslayer::api::{FItemType, FMeta, OwnedFlatFItem, OwnedFlatFTree};
use rocket::http::Status;
use rocket::response::{status, Responder};
use rocket::Request;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Deserialize, Serialize)]
pub enum FType {
    File(String),
    Folder,
}

impl From<FType> for FItemType {
    fn from(value: FType) -> Self {
        match value {
            FType::Folder => FItemType::Folder,
            FType::File(_) => FItemType::File,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct FNode {
    pub name: String,
    pub f_type: FType,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct FTree {
    pub inner: FNode,
    pub children: Vec<FTree>,
}

impl Default for FTree {
    fn default() -> Self {
        Self::new()
    }
}

impl FTree {
    pub fn new() -> FTree {
        Self {
            inner: FNode {
                name: "root".to_string(),
                f_type: FType::Folder,
            },
            children: Vec::new(),
        }
    }
    pub fn traverse_to_path(&self, path: &[&str]) -> Option<&FTree> {
        let mut cur_root = self;

        //Check if the given path exists
        //Short Circuit Return if it doesn't
        for i in path {
            cur_root = cur_root.children.iter().find(|f| f.inner.name == *i)?;
        }
        Some(cur_root)
    }

    pub fn traverse_to_path_mut(&mut self, path: &[&str]) -> Option<&mut FTree> {
        let mut cur_root = self;

        //Check if the given path exists
        //Short Circuit Return if it doesn't
        for i in path {
            cur_root = cur_root.children.iter_mut().find(|f| f.inner.name == *i)?;
        }
        Some(cur_root)
    }
}


impl From<FTree> for OwnedFlatFItem {
    fn from(value: FTree) -> Self {
        OwnedFlatFItem {
            name: value.inner.name,
            entry_type: value.inner.f_type.into(),
        }
    }
}
impl From<FTree> for OwnedFlatFTree {
    fn from(value: FTree) -> Self {
        OwnedFlatFTree {
            meta: FMeta {
                name: value.inner.name,
            },
            entry_type: value.inner.f_type.into(),
            sub_entries: value.children.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait]
pub trait FSHandle {
    async fn get(&self) -> Result<FNode, FSError>;

    async fn list_children(&self) -> Result<Vec<FNode>, FSError>;

    async fn tree(&self) -> Result<FTree, FSError>;

    async fn change_head(&mut self, path: &[&str]) -> Result<(), FSError>;

    async fn create_entry(&mut self, entry: FNode) -> Result<FTree, FSError>;

    //TODO delete_entry
}

#[derive(Error, Debug)]
pub enum FSError {
    #[error("Path Not Found")]
    PathNotFound,
    #[error("Operation Failed: {0}")]
    OperationFailed(String),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for FSError {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        match self {
            FSError::PathNotFound => status::NotFound("Path Not Found").respond_to(request),
            FSError::OperationFailed(x) => status::Custom(
                Status::InternalServerError,
                format!("Operation Failed: {x}"),
            )
            .respond_to(request),
        }
    }
}
