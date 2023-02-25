use std::collections::HashMap;

use jsonwebtoken::{jwk::{self, AlgorithmParameters}, decode_header, DecodingKey, Validation, decode};
use rocket::{serde::{Serialize, Deserialize, json}, Request, http::Status};
use rocket::request::{Outcome, FromRequest};

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

pub struct JWTAuth<'r>(&'r str);

#[derive(Debug)]
pub enum JWTError {
    Missing,
    Invalid,
}


#[rocket::async_trait]
impl<'r> FromRequest<'r> for JWTAuth<'r> {
    type Error = JWTError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        /// Returns true if `key` is a valid API key string.
        fn is_valid(bearerkey: &str) -> bool {
// http://localhost:8080/realms/garage-dev/protocol/openid-connect/certs
            const JWKS_REPLY: &str = r#"
{"keys":[{
    "kid": "QAsGatZf-DoSh5Sg2S8l-kBF9P4cAbVjJcZrp_r-mKw",
    "kty": "RSA",
    "alg": "RS256",
    "use": "sig",
    "n": "23YaSMGqVj3LnEtSz2YhANFvwoL3hTEkFinLSCeTcij2XOz1a1WkhsQZD0PR_N5ZFRqVThxh4sqO2pkoGdEPEO7MJUaKldroFc3vDOQAmhegVyd6zhUHJAwZ_7iGTMa2mEHJ9_OZWnvMY1g7Dk6Da5XAVaiaxTog7qLbO4jLeBUgVPRyxJya13sp_M_ME7MabCDK4H9S7Inf5MdqZcaUTTYlG41oYfVt3xX6bKZyP2SAbCuXqNj1bcB-ykTtHXuZSz7IKyRN_ObBfBZztzoEyQpXWMBmcM7VGE2oeZ_bNWdvrrv3SJYvCcR6C6tlGfw-7iIvxJ47oTCfC992rBkxew",
    "e": "AQAB",
    "x5c": [
      "MIICozCCAYsCBgGGZu1U9zANBgkqhkiG9w0BAQsFADAVMRMwEQYDVQQDDApnYXJhZ2UtZGV2MB4XDTIzMDIxODIzNDYxNVoXDTMzMDIxODIzNDc1NVowFTETMBEGA1UEAwwKZ2FyYWdlLWRldjCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBANt2GkjBqlY9y5xLUs9mIQDRb8KC94UxJBYpy0gnk3Io9lzs9WtVpIbEGQ9D0fzeWRUalU4cYeLKjtqZKBnRDxDuzCVGipXa6BXN7wzkAJoXoFcnes4VByQMGf+4hkzGtphByffzmVp7zGNYOw5Og2uVwFWomsU6IO6i2zuIy3gVIFT0csScmtd7KfzPzBOzGmwgyuB/UuyJ3+THamXGlE02JRuNaGH1bd8V+mymcj9kgGwrl6jY9W3AfspE7R17mUs+yCskTfzmwXwWc7c6BMkKV1jAZnDO1RhNqHmf2zVnb66790iWLwnEegurZRn8Pu4iL8SeO6EwnwvfdqwZMXsCAwEAATANBgkqhkiG9w0BAQsFAAOCAQEAcZ1aQZ3G74R4GtOS+EC8Sz+GlDd7XVChuxNp7zxOjb0DFlsGl9oAfJKwbsRfB2hpbzahCsgrENb0pehulWAMmZ9iKT9esKoQwAT1RJb/YpjtWVdTZTJSYKEu5Kc2QNEC2jcpfXFxovD0EFIz3LLCTOhBJrFitJSBS95hT9Ufnec1w0UzcqCa/cyI3hFDNyso8JMFy+a2obCJztRj7VEogfchu1oc1Crzzi65/KxXKy4n1R0GN/2FG8Iuj5SojpEsoQNX36RoCnmbxNUFmg300E4AE+f+wOFmif/8+FJgiSkHOWwoLhJDy4IO5khppjFEQxX5rhSCl2k9onc4JJmSHA=="
    ],
    "x5t": "EgQjhO0Y4jBNGGXeIDl1myogs-8",
    "x5t#S256": "YUnNbrOx7UTzSrLqoxaOGxzu6cBMBLWpPlYoN8C0XYQ"
  }]}
"#;         let key = bearerkey.replace("Bearer ", "");

            let jwks: jwk::JwkSet = json::from_str(JWKS_REPLY).unwrap();

            let header = match decode_header(&key) {
                Err(_e) => return false,
                Ok(t) => t,
            };
            let kid = match header.kid {
                Some(k) => k,
                None => return false,
            };
            if let Some(j) = jwks.find(&kid) {
                match &j.algorithm {
                    AlgorithmParameters::RSA(rsa) => {
                        let decoding_key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e).unwrap();
                        let mut validation = Validation::new(j.common.algorithm.unwrap());
                        validation.validate_exp = false;
                        let decoded_token =
                            decode::<HashMap<String, json::Value>>(&key, &decoding_key, &validation)
                                .unwrap();
                        println!("{:?}", decoded_token.claims);
                    }
                    _ => unreachable!("this should be a RSA"),
                }
            } else {
                return false
            }
            true
        }

        match req.headers().get_one("authorization") {
            None => Outcome::Failure((Status::BadRequest, JWTError::Missing)),
            Some(key) if is_valid(key) => Outcome::Success(JWTAuth(key)),
            Some(_) => Outcome::Failure((Status::BadRequest, JWTError::Invalid)),
        }
    }
}