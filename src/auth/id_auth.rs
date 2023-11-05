use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

// use fancy_regex::Regex;
// use lazy_static::lazy_static;

// lazy_static! {
//     static ref RE: Regex =
//         Regex::new(r"^(?:[[a-fA-F0-9]]{2}([-:]))(?:[[a-fA-F0-9]]{2}\1){4}[[a-fA-F0-9]]{2}$")
//             .unwrap();
// }

pub struct ClientId(pub String);

#[derive(Debug)]
pub enum ClientIdError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientId {
    type Error = ClientIdError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = request.headers().get_one("Authorization");
        match token {
            None => Outcome::Failure((Status::Forbidden, Self::Error::Missing)),
            // Some(t) => match RE.is_match(t) {
            //     Ok(t) if t => Outcome::Success(ClientId(t.to_string())),
            //     _ => Outcome::Failure((Status::BadRequest, Self::Error::Invalid)),
            // },
            Some(t) => Outcome::Success(ClientId(t.to_string())),
        }
    }
}
