use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::ops::Deref;

pub fn cannonicalise<'a>(comps: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut stack = Vec::new();
    for t in comps.map(&str::trim) {
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

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct FMeta {
    name: String,
    //In prod it should be file_path
    // content: String,
    file_path: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub enum FType {
    #[default]
    Folder,
    File,
}

impl FType {
    pub fn is_folder(&self) -> bool {
        matches!(self, FType::Folder)
    }

    pub fn is_file(&self) -> bool {
        matches!(self, FType::File)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FTree {
    meta: FMeta,
    sub_entries: HashMap<String, FTree>,
    entry_type: FType,
}

impl FTree {

    pub fn get_name(&self) -> &str {
        self.meta.name.as_str()
    }

    pub fn set_file_path(&mut self, f_path: &str) {
        self.meta.file_path = f_path.to_string();
    }

    pub fn create_sub_tree<'a>(
        &mut self,
        path: &mut impl Iterator<Item = impl Deref<Target = &'a str>>,
        f_type: FType
    ) -> &mut FTree {
        if let Some(t) = path.next() {
            if !self.sub_entries.contains_key(*t) {
                //Insert key
                self.sub_entries.insert((*t).to_string(), FTree {
                    sub_entries: HashMap::new(),
                    meta: FMeta {
                        name: t.to_string(),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            let q = self.sub_entries.get_mut(*t).expect("Value Inserted but not present?");
            q.create_sub_tree(path, f_type)
        } else {
            self.entry_type = f_type;
            self
        }
    }

    pub fn traverse<'a>(
        &self,
        comps: &mut impl Iterator<Item = impl Deref<Target = &'a str>>,
    ) -> Option<&FTree> {
        if let Some(t) = comps.next() {
            if let Some(q) = self.sub_entries.get(*t) {
                q.traverse(comps)
            } else {
                None
            }
        } else {
            Some(self)
        }
    }

    pub fn traverse_mut<'a>(
        &mut self,
        mut comps: impl Iterator<Item = &'a str>,
    ) -> Option<&mut FTree> {
        if let Some(t) = comps.next() {
            if let Some(q) = self.sub_entries.get_mut(t) {
                q.traverse_mut(comps)
            } else {
                None
            }
        } else {
            Some(self)
        }
    }

    pub fn flatten(&self) -> FlatFTree {
        FlatFTree {
            meta: &self.meta,
            sub_entries: self.sub_entries.values().map(Into::into).collect(),
            entry_type: self.entry_type,
        }
    }

    pub fn flatten_owned(&self) -> OwnedFlatFTree {
        OwnedFlatFTree {
            meta: self.meta.clone(),
            entry_type: self.entry_type,
            sub_entries: self.sub_entries.values().map(Into::into).collect(),
        }
    }

    pub fn is_file(&self) -> bool {
        self.entry_type.is_file()
    }

    pub fn is_folder(&self) -> bool {
        self.entry_type.is_folder()
    }

    pub fn get_file_path(&self) -> &str {
        self.meta.file_path.as_str()
    }
}

impl FTree {
    pub fn new_folder(name: &str) -> FTree {
        FTree {
            meta: FMeta {
                name: name.to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn add_file(&mut self, fname: &str, file_path: &str) {
        self.sub_entries.insert(
            fname.to_string(),
            FTree {
                meta: FMeta {
                    name: fname.to_string(),
                    file_path: file_path.to_string(),
                },
                entry_type: FType::File,
                ..Default::default()
            },
        );
    }

    pub fn add_folder(&mut self, fname: &str) -> &mut FTree {
        self.sub_entries.insert(
            fname.to_string(),
            FTree {
                meta: FMeta {
                    name: fname.to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        self.sub_entries.get_mut(fname).unwrap()
    }
}

#[derive(Debug, Serialize)]
pub struct FlatFTree<'a> {
    meta: &'a FMeta,
    sub_entries: Vec<FlatFItem<'a>>,
    entry_type: FType,
}

#[derive(Debug, Serialize)]
pub struct FlatFItem<'a> {
    name: &'a str,
    entry_type: FType,
}

impl<'a> From<&'a FTree> for FlatFItem<'a> {
    fn from(value: &'a FTree) -> Self {
        Self {
            name: &value.meta.name,
            entry_type: value.entry_type,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OwnedFlatFItem {
    name: String,
    entry_type: FType,
}

impl From<&FTree> for OwnedFlatFItem {
    fn from(value: &FTree) -> Self {
        OwnedFlatFItem {
            entry_type: value.entry_type,
            name: value.meta.name.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OwnedFlatFTree {
    meta: FMeta,
    sub_entries: Vec<OwnedFlatFItem>,
    entry_type: FType,
}
