use serde::Deserialize;
use serde::Serialize;
use std::borrow::Borrow;
use std::collections::HashMap;


pub fn cannonicalise<'a>(comps: impl Iterator<Item=&'a str>) -> Vec<&'a str>{
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
    return stack;
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FMeta {
    name: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub enum FType {
    #[default]
    Folder,
    File
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FTree {
    //should 
    meta: FMeta,
    sub_entries: HashMap<String, FTree>,
    entry_type: FType
}

impl FTree {

    pub fn new_folder(name: &str) -> FTree {
        FTree { meta: FMeta { name: name.to_string() }, ..Default::default() }
    }

    pub fn traverse<'a>(&self, mut comps: impl Iterator<Item=&'a str>) -> Option<&FTree> {
        //impl lc algo
        if let Some(t) = comps.next() {
            if let Some(q) = self.sub_entries.get(t) {
                q.traverse(comps)
            } else {
                None
            }
        } else {
            Some(self)
        }
    }

    pub fn traverse_mut<'a>(&mut self, comps: impl Iterator<Item = &'a str>) -> &mut FTree {
        //impl lc algo
        for t in comps {
            println!("{}", t)
        }
        self
    }

    pub fn flatten(&self) -> FlatFTree {
        FlatFTree { meta: &self.meta, sub_entries: self.sub_entries.values().map(|f| f.into()).collect(), entry_type: self.entry_type }
    }

    pub fn add_file(&mut self, fname: &str) {
        self.sub_entries.insert(fname.to_string(), FTree {
            meta: FMeta { name: fname.to_string() },
            entry_type: FType::File,
            ..Default::default()
        });
    }

    pub fn add_folder(&mut self, fname: &str) -> &mut FTree {
        self.sub_entries.insert(fname.to_string(), FTree {
            meta: FMeta { name: fname.to_string() },
            ..Default::default()
        });
        self.sub_entries.get_mut(fname).unwrap()    
    }

}

#[derive(Debug, Serialize)]
pub struct FlatFTree<'a> {
    meta: &'a FMeta,
    sub_entries: Vec<FlatFItem<'a>>,
    entry_type: FType
}

#[derive(Debug, Serialize)]
pub struct FlatFItem<'a> {
    name: &'a str,
    entry_type: FType
}

impl<'a> From<&'a FTree> for FlatFItem<'a> {
    fn from(value: &'a FTree) -> Self {
        Self { name: &value.meta.name, entry_type: value.entry_type }
    }
}

pub struct PathSegment {
    pub(crate) part: String,
}

pub trait FileHandle {}

pub trait DirectoryHandle {}

pub enum FsOption {
    File(Box<dyn FileHandle>),
    Directory(Box<dyn DirectoryHandle>),
}

impl FsOption {
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    pub fn is_directory(&self) -> bool {
        matches!(self, Self::Directory(_))
    }
}

pub trait FsProvider {
    fn navigate(&mut self, to: &dyn Borrow<PathSegment>);

    fn fetch(&self) -> FsOption;

    fn mkdir(&mut self, name: &str) -> Box<dyn DirectoryHandle>;

    fn create_file(&mut self, name: &str) -> Box<dyn FileHandle>;

    fn pwd(&self) -> &[PathSegment];
}
