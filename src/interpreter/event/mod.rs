use crate::interpreter::object::Object;

#[derive(Clone, Debug, Default)]
pub enum InterpreterEvent {
    Render(Object),
    #[default]
    None,
}
