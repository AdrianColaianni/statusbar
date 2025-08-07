use std::time::Duration;

pub mod volume;
pub mod battery;
pub mod time;
pub mod internet;

pub trait Block {
    fn new() -> Self where Self: Sized;
    fn frequency(&self) -> Duration;
    fn update(&mut self) -> bool;
    fn get_text(&self) -> String;
}
