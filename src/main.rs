use std::fs::{File, OpenOptions};
use std::io::{BufReader, ErrorKind, Write};
use std::ops::Deref;
use std::sync::Arc;

use rocket::tokio::sync::RwLock;

use crate::fslayer::afs::{FNode, FSError, FSHandle, FTree, FType};
use crate::fslayer::native::hashmapfs::InMemoryFSHandle;
use fslayer::api::{OwnedFlatFItem, OwnedFlatFTree};
use rocket::data::ToByteUnit;
use rocket::http::uri::fmt::Path;
use rocket::serde::json::serde_json;
use rocket::{
    fs::{FileServer, NamedFile},
    http::uri::Segments,
    serde::json::Json,
    Data, State,
};
use utils::cannonicalise;
use uuid::Uuid;

const UPLOAD_DIR: &str = "upload";

pub mod fslayer;
pub mod utils;

#[macro_use]
extern crate rocket;

type RwTree = Arc<RwLock<FTree>>;

fn reflect_tree(value: &FTree) -> Result<(), serde_json::Error> {
    serde_json::to_writer(
        OpenOptions::new()
            .write(true)
            .open("fsindex.json")
            .expect("Should be present"),
        value,
    )
}

#[get("/<_..>")]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("dist/index.html").await
}

#[get("/<path..>")]
async fn get_entry(
    path: Segments<'_, Path>,
    st: &State<RwTree>,
) -> Result<Json<OwnedFlatFTree>, FSError> {
    let pa = cannonicalise(path);
    let mut manager: InMemoryFSHandle = st.deref().clone().into();
    manager.change_head(pa.as_slice()).await?;
    let res = manager.tree().await?;
    Ok(Json(res.into()))
}

#[put("/<path..>", data = "<item>")]
async fn create_entry(
    path: Segments<'_, Path>,
    st: &State<RwTree>,
    mut item: Data<'_>,
) -> Result<Json<OwnedFlatFItem>, FSError> {
    let pa = cannonicalise(path);
    let f_type = if item.peek(1).await.is_empty() {
        //treat entry as folder
        FType::Folder
    } else {
        let mut new_file_name = Uuid::new_v4().to_string();

        let file_name_in_path = pa[pa.len() - 1];
        let extension_position = utils::rfind_utf8(file_name_in_path, '.');

        if let Some(pos) = extension_position {
            new_file_name.push_str(&file_name_in_path[pos..]);
        }

        let file_path = std::path::Path::new(UPLOAD_DIR).join(new_file_name);

        FType::File(
            file_path
                .to_str()
                .expect("Should be all safe UTF here")
                .to_string(),
        )
    };

    let mut manager: InMemoryFSHandle = st.deref().clone().into();
    manager
        .change_head(&pa.as_slice()[0..(pa.len() - 1)])
        .await?;

    let res = manager
        .create_entry(FNode {
            f_type,
            name: pa[pa.len() - 1].to_string(),
        })
        .await?;

    if let FType::File(x) = &res.inner.f_type {
        let file_str_res = item.open(25.mebibytes()).into_file(x).await;
        if file_str_res.is_err() {
            return Err(FSError::OperationFailed("Persistence Failed".to_string()));
        }

        let tree = manager.inner.read().await;

        if reflect_tree(&tree).is_err() {
            return Err(FSError::OperationFailed("Persistence Failed".to_string()));
        }
    }

    Ok(Json(res.into()))
}

#[get("/<path..>")]
async fn get_raw<'a>(path: Segments<'a, Path>, st: &State<RwTree>) -> Result<NamedFile, FSError> {
    let pa = cannonicalise(path);
    let mut manager: InMemoryFSHandle = st.deref().clone().into();
    manager.change_head(pa.as_slice()).await?;
    let res = manager.get().await?;
    if let FType::File(x) = res.f_type {
        Ok(NamedFile::open(x).await.unwrap())
    } else {
        Err(FSError::PathNotFound)
    }
}

// #[get("/<path..>")]
// fn get_collab<'a>(
//     path: Segments<Path>,
//     st: &'a State<FTree>,
// ) -> Result<Json<FItemContent<'a>>, NotFound<String>> {
//     let pa = cannonicalise(path);
//     let res = st.traverse(&mut pa.iter());
//     if let Some(tree) = res {
//         if tree.is_file() {
//            return Ok(Json(tree.get_content()));
//         }
//     }
//     return Err(NotFound(format!("No content at path: {} ", pa.join("/"))));
// }

#[launch]
fn rocket() -> _ {
    std::fs::create_dir_all(UPLOAD_DIR).expect("Could not create upload dir");
    let index_file = File::open("fsindex.json");
    let index_tree = match index_file {
        Ok(f) => {
            serde_json::from_reader::<_, FTree>(BufReader::new(f)).expect("Index File corrupted")
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let default_index = FTree::new();
            let mut f = File::create("fsindex.json").expect("Could not create index file");
            reflect_tree(&default_index).expect("Could not write to index file");
            let _ = f.flush();
            default_index
        }
        _ => {
            panic!("Unknown error {index_file:?}")
        }
    };
    let state = Arc::new(RwLock::new(index_tree));
    rocket::build()
        .manage(state)
        .mount("/f", routes![index])
        .mount("/api", routes![get_entry, create_entry])
        .mount("/raw", routes![get_raw])
        // .mount("/collab", routes![get_collab])
        .mount("/", FileServer::from("dist/"))
}
