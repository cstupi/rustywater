struct JWT(String);

#[derive(Debug)]
enum JWTError {
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for JWT {
    type Error = JWTError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let token = request.headers.get_one("authorization");
        match token {
            some(token) => {
                Outcome::Success(JWT(token.to_string()))
            }
            None => Outcome::Failure((Status::Unauthorized, JWTError::Missing))
        }
    }
}