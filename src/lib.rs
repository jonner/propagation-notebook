use toasty::ModelSet;

pub mod collecting;
pub mod propagation;
pub mod region;
pub mod taxonomy;

pub fn models() -> ModelSet {
    toasty::models!(crate::*)
}
