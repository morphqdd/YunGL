use crate::interpreter::object::Object;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug, Default)]
pub enum InterpreterEvent {
    Render(Object),
    #[default]
    None,
    GetWindowDimensions(Sender<(u32, u32)>),
}
