#[macro_use] extern crate rocket;

use gpio::{GpioOut};
use jsonwebtoken::jwk::AlgorithmParameters;
use jsonwebtoken::{decode, Validation, DecodingKey, decode_header, jwk};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::request::{Outcome, FromRequest};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::{json};
use rocket::{State, fairing::AdHoc};
use std::collections::HashMap;
use std::thread::spawn;
use std::thread::sleep;
use std::time::Duration;
use rocket::{Request, Response};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Config {
    gpio_enabled: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

struct ApiKey<'r>(&'r str);

#[derive(Debug)]
enum ApiKeyError {
    Missing,
    Invalid,
}


#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

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
            None => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) if is_valid(key) => Outcome::Success(ApiKey(key)),
            Some(_) => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
        }
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

#[get("/checkauth")]
fn checkauth(_token: ApiKey<'_>) -> &'static str {
    "Hello, world!"
}


#[put("/<pin>/toggle/<enable>")]
fn toggle(pin: u16, enable: bool, config: &State<Config>) {
    if !config.gpio_enabled {
        return
    }
    let mut gpio = gpio::sysfs::SysFsGpioOutput::open(pin).unwrap();
    gpio.set_value(enable).expect("could not set gpio4");
}

#[put("/<pin>/blink/<interval>/count/<count>")]
fn blink(pin: u16, interval: u64, count: u64, config: &State<Config>) {
    if !config.gpio_enabled {
        return
    }
    spawn(move || {
        let mut gpio = gpio::sysfs::SysFsGpioOutput::open(pin).unwrap();
        for _i in 1..count {
            gpio.set_value(true).expect("could not set pin");
            sleep(Duration::from_secs(interval));
            gpio.set_value(false).expect("could not set pin");
        }
    });
}

// Webhook compatible way to turn on a pin for a time
#[get("/<pin>/timed/<time>")]
fn timed(pin: u16, time: u64, config: &State<Config>) {
    println!("GPIO ENABLED: {}", config.gpio_enabled);
    if !config.gpio_enabled {
        return
    }
    spawn(move || {
        let mut gpio = gpio::sysfs::SysFsGpioOutput::open(pin).unwrap();
        gpio.set_value(true).expect("could not set pin");
        sleep(Duration::from_secs(time));
        gpio.set_value(false).expect("could not set pin");
        println!("Pin: {}, Time: {}", pin, time);
    });
}

#[launch]
fn rocket() -> _ {

    rocket::build()
    .mount("/", routes![index, checkauth, all_options])
    .mount("/pin", routes![toggle, timed, blink])
    .attach(AdHoc::config::<Config>())
    .attach(CORS)
}