use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Status {
    code: u8,
    msg: String,

}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response<T> {
    code: u8,
    msg: String,
    data: T,
}


impl Status {
    pub fn success() -> Self {
        Self {
            code: 1,
            msg: String::from("success"),

        }
    }

    pub fn fail() -> Self {
        Self {
            code: 0,
            msg: String::from("fail"),
        }
    }

    pub fn fail_with_msg(info: String) -> Self {
        Self {
            code: 0,
            msg: info,
        }
    }

    pub fn data<T>(t: T) -> Response<T> {
        Response {
            code: 1,
            msg: String::from("success"),
            data: t,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sector {
    pub id: u32,
    pub title: String,
    pub content: String,
}