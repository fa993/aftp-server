use std::fs::{File, OpenOptions};
use std::io::{BufReader, ErrorKind, Write};
use std::sync::Arc;

use rocket::tokio::sync::RwLock;

use fslayer::fsprovider::cannonicalise;
use rocket::serde::json::serde_json;
use rocket::{fs::{FileServer, NamedFile}, http::uri::{Segments}, response::status::NotFound, serde::json::Json, Data, State, Either};
use rocket::data::ToByteUnit;
use rocket::http::Status;
use rocket::http::uri::fmt::Path;
use uuid::Uuid;

use crate::fslayer::fsprovider::{FTree, FType, OwnedFlatFItem, OwnedFlatFTree};

const UPLOAD_DIR: &str = "upload";

pub mod fslayer;

#[macro_use]
extern crate rocket;

type RwTree = Arc<RwLock<FTree>>;

fn rfind_utf8(s: &str, chr: char) -> Option<usize> {
    if let Some(rev_pos) = s.chars().rev().position(|c| c == chr) {
        Some(s.chars().count() - rev_pos - 1)
    } else {
        None
    }
}

fn reflect_tree(value: &FTree) -> Result<(), serde_json::Error>{
    serde_json::to_writer(OpenOptions::new().write(true).open("fsindex.json").expect("Should be present"), value)
}

#[get("/<_..>")]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("dist/index.html").await
}

#[get("/<path..>")]
async fn get_entry(
    path: Segments<'_, Path>,
    st: &State<RwTree>,
) -> Result<Json<OwnedFlatFTree>, NotFound<String>> {
    let pa = cannonicalise(path);
    let st = st.read().await;
    let res = st.traverse(&mut pa.iter());
    if let Some(tree) = res {
        Ok(Json(tree.flatten_owned()))
    } else {
        Err(NotFound(format!("No entry at path: {} ", pa.join("/"))))
    }
}

#[put("/<path..>", data = "<item>")]
async fn create_entry(
    path: Segments<'_, Path>,
    st: &State<RwTree>,
    mut item: Data<'_>,
) -> Result<Json<OwnedFlatFItem>, Status> {
    let f_type = if item.peek(1).await.len() == 0 {
        //treat entry as folder
        FType::Folder
    } else {
        FType::File
    };
    let pa = cannonicalise(path);
    let mut st = st.write().await;

    let res = st.create_sub_tree(&mut pa.iter(), f_type);

    let final_tree = if f_type.is_file() {
        let mut new_file_name = Uuid::new_v4().to_string();
        let file_name_in_path = res.get_name();
        let extension_position = rfind_utf8(file_name_in_path, '.');
        if let Some(pos) = extension_position {
            new_file_name.push_str(&file_name_in_path[pos..]);
        }
        let file_path = std::path::Path::new(UPLOAD_DIR).join(new_file_name);
        res.set_file_path(file_path.to_str().expect("Should be all safe UTF here"));


        let file_str_res = item.open(25.mebibytes()).into_file(file_path).await;
        if file_str_res.is_err() {
            return Err(Status::InternalServerError);
        }

        let output_res = (&*res).into();

        if reflect_tree(&*st).is_err() {
            return Err(Status::InternalServerError);
        }

        output_res
    } else {
        (&*res).into()
    };
    Ok(Json(final_tree))
}

#[get("/<path..>")]
async fn get_raw<'a>(
    path: Segments<'a, Path>,
    st: &State<RwTree>,
) -> Result<NamedFile, NotFound<String>> {
    let pa = cannonicalise(path);
    let st = st.read().await;
    let res = st.traverse(&mut pa.iter());
    if let Some(tree) = res {
        if tree.is_file() {
            return Ok(NamedFile::open(tree.get_file_path()).await.unwrap());
        }
    }
    return Err(NotFound(format!("No content at path: {} ", pa.join("/"))));
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
            let default_index = FTree::new_folder("root");
            let mut f = File::create("fsindex.json").expect("Could not create index file");
            reflect_tree(&default_index).expect("Could not write to index file");
            let _ = f.flush();
            default_index
        }
        _ => {
            panic!("Unknown error {:?}", index_file)
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
