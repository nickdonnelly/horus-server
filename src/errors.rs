#[derive(Debug)]
pub enum AuthTokenError
{
    Invalid,
    Expired,
    ConsumeFailure,
    NotFound
}
