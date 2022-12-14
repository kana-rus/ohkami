use crate::response::format::ResponseFormat;


#[derive(Debug, PartialEq)]
pub enum Status {
    OK                  = 200,
    Created             = 201,
    NoContent           = 204,
    BadRequest          = 400,
    Unauthorized        = 401,
    Forbidden           = 403,
    NotFound            = 404,
    InternalServerError = 500,
    NotImplemented      = 501,
}

impl ResponseFormat for Status {
    fn response_format(&self) -> &'static str {
        match self {
            Self::BadRequest          => "400 Bad Request",
            Self::InternalServerError => "500 Internal Server Error",
            Self::NotFound            => "404 Not Found",
            Self::Forbidden           => "403 Forbidden",
            Self::Unauthorized        => "401 Unauthorized",
            Self::NotImplemented      => "501 Not Implemented",
            Self::OK                  => "200 OK",
            Self::Created             => "201 Created",
            Self::NoContent           => "204 No Content",
        }
    }
}