// C:\Users\nasty\polivoks_rs\src\effects\mod.rs
pub mod spring_reverb;

pub use spring_reverb::SovietSpringReverb;

pub trait Effect {
    fn process(&mut self, input: f32, dt: f32) -> f32;
    fn set_parameter(&mut self, param: usize, value: f32);
    fn get_parameter(&self, param: usize) -> f32;
    fn get_parameter_name(&self, param: usize) -> &'static str;
    fn get_name(&self) -> &'static str;
}