use wayland_client::Connection;
use wayland_imf::{
    keyboard::KeyboardConfig,
    state::{Config, State},
};

fn parse_config_from<I>(args: I, env_layout: Option<String>) -> Result<Config, String>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let mut layout = env_layout;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--layout" => {
                let value = args
                    .next()
                    .ok_or_else(|| "`--layout` requires a value".to_string())?;
                layout = Some(value);
            }
            "--help" | "-h" => {
                return Err("help".to_string());
            }
            other => return Err(format!("Unknown argument: {other}")),
        }
    }

    Ok(Config {
        keyboard: KeyboardConfig { layout },
    })
}

fn parse_config() -> Config {
    match parse_config_from(
        std::env::args().skip(1),
        std::env::var("WAYLAND_IMF_KEYBOARD_LAYOUT").ok(),
    ) {
        Ok(config) => config,
        Err(err) if err == "help" => {
            println!("Usage: wayland-imf [--layout <xkb-layout>]");
            println!("Env: WAYLAND_IMF_KEYBOARD_LAYOUT=<xkb-layout>");
            std::process::exit(0);
        }
        Err(err) => panic!("{err}"),
    }
}

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let mut state = State::new(parse_config());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_layout_overrides_env_layout() {
        let config = parse_config_from(
            vec!["--layout".to_string(), "us".to_string()],
            Some("jp".to_string()),
        )
        .unwrap();

        assert_eq!(config.keyboard.layout.as_deref(), Some("us"));
    }

    #[test]
    fn env_layout_is_used_when_cli_is_absent() {
        let config = parse_config_from(Vec::<String>::new(), Some("jp".to_string())).unwrap();

        assert_eq!(config.keyboard.layout.as_deref(), Some("jp"));
    }
}
