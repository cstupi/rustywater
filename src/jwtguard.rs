use std::{collections::HashMap, sync::Arc};

use jsonwebtoken::{jwk::{self, AlgorithmParameters}, decode_header, DecodingKey, Validation, decode};
use rocket::{serde::{Serialize, Deserialize, json}, Request, http::{Status, ext::IntoCollection}, State, tokio::sync::RwLock};
use rocket::request::{Outcome, FromRequest};

use crate::jwkstore::{JwkStore};

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

pub struct JwtGuard<'r>(&'r str);

#[derive(Debug)]
pub enum JWTError {
    Missing,
    Invalid,
}

fn validate_jwt(jwk_store: &JwkStore, jwt: &str) -> bool {
    let jwt_header = match decode_header(&jwt) {
        Err(_e) => return false,
        Ok(t) => t,
    };
    let kid = match jwt_header.kid {
        Some(k) => k,
        None => return false,
    };

    match jwk_store.jwks.find(&kid).cloned() {
        Some(jwk) => {
            match &jwk.algorithm {
                AlgorithmParameters::RSA(rsa) => {
                    let decoding_key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e).unwrap();
                    let mut validation = Validation::new(jwk.common.algorithm.unwrap());
                    validation.validate_exp = true;
                    println!("Validation algo");
                    match decode::<HashMap<String, json::Value>>(&jwt, &decoding_key, &validation) {
                        Ok(decoded_token) => {
                            println!("Successful decoded token");
                            true 
                        },
                        Err(err) => {
                            println!("Error decoding token: {}", err);
                            false
                        },
                    }
                }
                _ => unreachable!("this should be a RSA"),
            
            }
        },
        None => {
            println!("jwk not found matching kid, {}", &kid);
            false
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtGuard<'r> {
    type Error = JWTError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        /// Returns true if `key` is a valid API key string.
        fn is_valid(bearerkey: &str, jwk_store: &JwkStore) -> bool {
            let key = bearerkey.replace("Bearer ", "");
            validate_jwt(jwk_store, &key)
        }

        let jwk_store = req.guard::<&State<JwkStore>>().await.expect("JWKs are required");
        match req.headers().get_one("authorization") {
            None => Outcome::Failure((Status::BadRequest, JWTError::Missing)),
            Some(key) if is_valid(key, jwk_store) => Outcome::Success(JwtGuard(key)),
            Some(_) => Outcome::Failure((Status::Unauthorized, JWTError::Invalid)),
        }
    }
}
