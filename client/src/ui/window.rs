use eframe::egui::Context;

use crate::state::RoomState;

pub trait Window {
    // Returns false if window has been closed
    fn show(&mut self, ctx: &Context, room_state: &mut RoomState) -> bool;
}
