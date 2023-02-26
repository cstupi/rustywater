
use jsonwebtoken::jwk;
use rocket::{fairing::{Fairing, Info, Kind}, Rocket, Orbit, serde::json};

pub struct JWKFairing {
  jwk_url: String,
  jwks: Option<jwk::JwkSet>,
}

impl JWKFairing {
  pub fn get_jwk(&self, kid: String) -> Option<jwk::Jwk> {
    self.jwks?.find(&kid).cloned()
  }
}

#[rocket::async_trait]
impl Fairing for JWKFairing {
    fn info(&self) -> Info {
        Info {
            name: "Store jwk from external source",
            kind: Kind::Singleton
        }
    }
    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
    //  let mut res = reqwest::blocking::get(self.jwk_url).expect("JWKSet not returned from external source");
      // let mut body = String::new();
      // let res_options = res.read_to_string(&mut body);
      // if !res_options {
      //   panic!("Failed to get jwk");
      // }


      let body = reqwest::get(self.jwk_url).await.expect("failed to call external source for jwkset")
      .text().await.expect("failed to get jwk request body");
      self.jwks = json::from_str(&body).expect("JWK did not deserialize");
    }
}