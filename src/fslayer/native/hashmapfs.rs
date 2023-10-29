use crate::fslayer::afs::{FNode, FSError, FSHandle, FTree};
use rocket::tokio::sync::RwLock;
use std::sync::Arc;

fn cannonicalise<'a>(comps: &'a [&'a str]) -> Vec<&'a str> {
    let mut stack = Vec::new();
    for t in comps {
        let t = t.trim();
        if t.is_empty() {
            continue;
        } else if t == ".." {
            stack.pop();
        } else if t == "." {
            continue;
        } else {
            stack.push(t);
        }
    }
    stack
}

pub struct InMemoryFSHandle {
    pub inner: Arc<RwLock<FTree>>,
    current_head: Vec<String>,
}

impl From<Arc<RwLock<FTree>>> for InMemoryFSHandle {
    fn from(value: Arc<RwLock<FTree>>) -> Self {
        Self {
            inner: value,
            current_head: Vec::new(),
        }
    }
}

impl InMemoryFSHandle {
    fn head(&self) -> Vec<&str> {
        self.current_head.iter().map(|f| f.as_str()).collect()
    }
}

#[async_trait]
impl FSHandle for InMemoryFSHandle {
    async fn get(&self) -> Result<FNode, FSError> {
        let root = self.inner.read().await;
        root.traverse_to_path(self.head().as_slice())
            .map(|f| f.inner.clone())
            .ok_or(FSError::PathNotFound)
    }

    async fn list_children(&self) -> Result<Vec<FNode>, FSError> {
        let root = self.inner.read().await;
        root.traverse_to_path(self.head().as_slice())
            .map(|f| f.children.iter().map(|f| f.inner.clone()).collect())
            .ok_or(FSError::PathNotFound)
    }

    async fn tree(&self) -> Result<FTree, FSError> {
        let root = self.inner.read().await;
        root.traverse_to_path(self.head().as_slice())
            .map(Clone::clone)
            .ok_or(FSError::PathNotFound)
    }

    async fn change_head(&mut self, path: &[&str]) -> Result<(), FSError> {
        // Check whether resource exist here
        // cost of traversal is far cheaper when everything is in memory
        let cn = cannonicalise(path);

        let root = self.inner.read().await;
        let cur_root = root
            .traverse_to_path(self.head().as_slice())
            .ok_or(FSError::PathNotFound)?;

        cur_root
            .traverse_to_path(path)
            .ok_or(FSError::PathNotFound)?;

        // Path Exists so extend Current Head
        self.current_head.extend(cn.iter().map(ToString::to_string));
        Ok(())
    }

    async fn create_entry(&mut self, entry: FNode) -> Result<FTree, FSError> {
        let mut root = self.inner.write().await;
        let cur_root = root
            .traverse_to_path_mut(self.head().as_slice())
            .ok_or(FSError::PathNotFound)?;
        //Check if same name already exists
        if cur_root.children.iter().any(|f| f.inner.name == entry.name) {
            Err(FSError::OperationFailed(
                "Entry with name already exists".to_string(),
            ))
        } else {
            let child = FTree {
                inner: entry,
                children: Vec::new(),
            };
            cur_root.children.push(child.clone());
            Ok(child)
        }
    }
}
