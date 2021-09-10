use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerMessage {
    pub addr: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatrixMessage {
    pub addr: String,
    pub matrix: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WheelMessage {
    pub addr: String,
    pub value: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinesMessage {
    pub addr: String,
    pub value: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatrixRequestMessage {
    addr: String,
}

impl MatrixRequestMessage {
    pub fn new() -> Self {
        Self {
            addr: String::from("/get-matrix"),
        }
    }
}

#[derive(Debug)]
pub enum Messages {
    Matrix(MatrixMessage),
    Wheel(WheelMessage),
    Lines(LinesMessage),
}
