use toasty::ModelSet;

pub mod collection;
pub mod protocol;
pub mod region;
pub mod taxonomy;

pub fn models() -> ModelSet {
    toasty::models!(crate::*)
}
