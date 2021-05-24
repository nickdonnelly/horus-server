use rocket::Request;
use rocket_contrib::templates::Template;

#[derive(Serialize)]
struct Context404 {
    uri: String
}

#[error(404)]
fn not_found(req: &Request) -> Template
{

    let context = Context404 {
        uri: req.uri().as_str().to_string()
    };

    Template::render("errors/404", &context)
}
