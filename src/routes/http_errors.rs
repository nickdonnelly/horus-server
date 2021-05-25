use rocket::Request;
use rocket_contrib::templates::Template;

#[derive(Serialize)]
struct Context404 {
    uri: String
}

#[catch(404)]
pub fn not_found(req: &Request) -> Template
{

    let context = Context404 {
        uri: req.uri().to_string()
    };

    Template::render("errors/404", &context)
}
