use fslayer::fsprovider::{cannonicalise, FItemContent};
use rocket::{
    fs::{FileServer, NamedFile},
    http::uri::{fmt::Path, Segments},
    response::status::NotFound,
    serde::json::Json,
    State,
};

use crate::fslayer::fsprovider::{FTree, FlatFTree};
use either::Either::{self, Left, Right};
pub mod fslayer;

#[macro_use]
extern crate rocket;

#[get("/<_..>")]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("dist/index.html").await
}

#[get("/<path..>")]
fn get_entry<'a>(
    path: Segments<Path>,
    st: &'a State<FTree>,
) -> Result<Either<Json<FItemContent<'a>>, Json<FlatFTree<'a>>>, NotFound<String>> {
    let pa = cannonicalise(path);
    let res = st.traverse(&mut pa.iter());
    if let Some(tree) = res {
        if tree.is_file() {
            Ok(Left(Json(tree.get_content())))
        } else {
            Ok(Right(Json(tree.flatten())))
        }
    } else {
        Err(NotFound(format!("No entry at path: {} ", pa.join("/"))))
    }
}

#[launch]
fn rocket() -> _ {
    let mut state = FTree::new_folder("root");
    state.add_file("hello.txt", "Hello World!");
    let mut sclv = state.add_folder("I");
    sclv.add_file("ok", "GG");
    sclv = sclv.add_folder("am");
    sclv = sclv.add_folder("thor");
    sclv.add_file("success.txt", "Congratulations!");
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
