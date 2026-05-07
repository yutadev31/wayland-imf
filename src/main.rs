use ime::state::State;
use wayland_client::Connection;

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let mut state = State::new();
    state.ime.init();

    let _registry = display.get_registry(&qh, ());

    event_queue.roundtrip(&mut state).unwrap();

    if let (Some(seat), Some(im_manager)) = (&state.wayland.seat, &state.wayland.im_manager) {
        let im = im_manager.get_input_method(seat, &qh, ());
        state.wayland.input_method = Some(im);
    }

    state.ensure_candidate_popup(&qh);

    if let (Some(seat), Some(vk_manager)) = (&state.wayland.seat, &state.wayland.vk_manager) {
        let vk = vk_manager.create_virtual_keyboard(seat, &qh, ());
        state.wayland.virtual_keyboard = Some(vk);
    }

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}
