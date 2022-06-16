use serde::{Deserialize, Deserializer};

/// Deserialize an optional type to default if none
pub fn value_or_default<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

macro_rules! into_request {
    ($type:ident) => {
        paste::paste! {
            impl From<[<$type Request>]> for crate::Request {
                fn from(msg: [<$type Request>]) -> Self {
                    let message = crate::Message::$type(msg);
                    Self { message }
                }
            }
            impl crate::RequestHandler for [<$type Request>] {}
        }
    };
}
pub(crate) use into_request;
