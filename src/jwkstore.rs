
use std::sync::Arc;

use jsonwebtoken::jwk;
use rocket::{fairing::{Fairing, Info, Kind}, Rocket, Orbit, serde::json, State};


pub struct JwkStore {
  pub jwks: jwk::JwkSet,
}

// pub struct JwkRetriever {
//   jwk_uri: String,
//   pub jwk_store: Option<JwkStore>
// }

// impl JwkRetriever {
//   pub fn new(jwk_uri: String) -> Self {
//       JwkRetriever { jwk_uri, jwk_store: None }
//   }
// }


// #[rocket::async_trait]
// impl Fairing for JwkRetriever {
//     fn info(&self) -> Info {
//         Info {
//             name: "Store jwk from external source",
//             kind: Kind::Liftoff
//         }
//     }
//     async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
//       let body = reqwest::get(&self.jwk_uri).await.expect("failed to call external source for jwkset")
//        .text().await.expect("failed to get jwk request body");
//       let jwks: jwk::JwkSet = json::from_str(&body).expect("JWK did not deserialize");
//       let jwk_store = JwkStore {
//         jwks: Arc::new(Some(jwks.clone())),
//       };
//       self.jwk_store = Some(jwk_store);
//       return
//     }
// }