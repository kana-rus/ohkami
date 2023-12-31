mod users;
mod user;
mod profiles;
mod articles;
mod tags;

use ohkami::{Ohkami, Route};
use crate::fangs::{LogRequest, LogResponse};


pub fn realworld_ohkami() -> Ohkami {
    Ohkami::with((LogRequest, LogResponse), (
        "/api/users"   .By(users::users_ohkami()),
        "/api/user"    .By(user::user_ohkami()),
        "/api/profiles".By(profiles::profiles_ohkami()),
        "/api/articles".By(articles::articles_ohkami()),
        "/api/tags"    .By(tags::tags_ohkami()),
    ))
}