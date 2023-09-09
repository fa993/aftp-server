use fslayer::fsprovider::cannonicalise;
use rocket::{serde::json::Json, State, http::uri::{Segments, fmt::Path}, fs::{NamedFile, FileServer}, response::status::NotFound};

use crate::fslayer::fsprovider::{FTree, FlatFTree};

pub mod fslayer;

#[macro_use]
extern crate rocket;

#[get("/<_..>")]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("dist/index.html").await
}

#[get("/<path..>")]
fn get_entry<'a>(path: Segments<Path>, st: &'a State<FTree>) -> Result<Json<FlatFTree<'a>>, NotFound<String>> {
    let pa = cannonicalise(path);
    let res = st.traverse(pa.clone().into_iter());
    if let Some(tree) = res {
        Ok(Json(tree.flatten()))
    } else {
        Err(NotFound(format!("No entry at path: {} ", pa.join("/"))))
    }
}

#[launch]
fn rocket() -> _ {
    let mut state = FTree::new_folder("root");
    state.add_file("hello.txt");
    let mut sclv = state.add_folder("I");
    sclv.add_file("ok");
    sclv = sclv.add_folder("am");
    sclv = sclv.add_folder("thor");
    sclv.add_file("success.txt");
    let uri = uri!("/a/b/.././c");
    for t in uri.path().segments() {
        println!("{t}")
    }
    rocket::build()
        .manage(state)
        .mount("/f", routes![index])
        .mount("/api", routes![get_entry])
        .mount("/", FileServer::from("dist/"))
}
