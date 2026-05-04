use ime::state::State;
use wayland_client::Connection;

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let mut state = State::new();

    let _registry = display.get_registry(&qh, ());

    event_queue.roundtrip(&mut state).unwrap();

    if let (Some(seat), Some(im_manager)) = (&state.seat, &state.im_manager) {
        let im = im_manager.get_input_method(seat, &qh, ());
        state.input_method = Some(im);
    }

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}
