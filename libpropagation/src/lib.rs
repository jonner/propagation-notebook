use toasty::ModelSet;

pub mod collecting;
pub mod propagation;
pub mod region;
pub mod taxonomy;

pub fn models() -> ModelSet {
    toasty::models!(crate::*)
}

pub trait ImportProgressReporter {
    fn begin_step(&mut self, name: &str, total: usize);
    fn increment(&mut self);
    fn finish_step(&mut self);
}
