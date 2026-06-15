use chrono::{prelude::*, Duration};
use log::{debug, error, info};
use rodio::{DeviceSinkBuilder, Player, Decoder};
use slint::{ModelRc, SharedString, VecModel};
use std::{cell::RefCell, io::BufReader, rc::Rc, thread};
use std::fs::File;
use std::path::PathBuf;

slint::include_modules!();

struct AppState {
    opts_time: Vec<DateTime<Local>>,

    slint_model: Rc<VecModel<SharedString>>,
}

fn compute_options() -> (Vec<DateTime<Local>>, Vec<SharedString>) {
    let now: DateTime<Local> = Local::now();
    debug!("compute_options at {}", now.format("%H:%M:%S"));

    let minutes_to_next_quarter = 15 - (now.minute() % 15);

    let first_target = (now + Duration::minutes(minutes_to_next_quarter as i64))
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();

    (0..8).map(|i| {
        let time = first_target + Duration::minutes(i * 15);
        let text: SharedString = time.format("%H:%M").to_string().into();

        (time, text)
    }).unzip()
}

/// Locate `game_over.mp3`, checking every place it can live across dev and the
/// packaged bundles. cargo-bundle / the PKGBUILD copy `assets/` to a
/// platform-specific dir:
///   - linux pkg: `/usr/lib/<name>/assets/...` (exe is `/usr/bin/<name>`)
///   - macOS app: `<App>.app/Contents/Resources/assets/...` (exe is `.../MacOS/<name>`)
///     In dev, fall back to the crate dir baked in at compile time.
fn sound_path() -> Option<PathBuf> {
    const REL: &str = "game_over.mp3";

    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // next to the executable
            candidates.push(dir.join("assets").join(REL));
            // linux pkg: /usr/bin/<name> -> /usr/lib/<name>/assets/<REL>
            candidates.push(dir.join("../lib/schisaciaccole/assets").join(REL));
            // macOS .app: Contents/MacOS/<name> -> Contents/Resources/assets/<REL>
            candidates.push(dir.join("../Resources/assets").join(REL));
        }
    }

    // dev: resolve from the crate root regardless of cwd
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets").join(REL));

    candidates.into_iter().find(|p| p.exists())
}

fn play_sound() {
    thread::spawn(|| {
        let Some(audio_path) = sound_path() else {
            error!("Sound file not found: game_over.mp3 (searched bundle + crate dirs)");
            return;
        };

        let mut sink_handle = DeviceSinkBuilder::open_default_sink().expect("Open default audio stream");
        sink_handle.log_on_drop(false);
        let player = Player::connect_new(sink_handle.mixer());

        match File::open(&audio_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                if let Ok(source) = Decoder::new(reader) {
                    info!("Playing sound: {}", audio_path.display());
                    player.append(source);
                    player.sleep_until_end();
                } else {
                    error!("Error while decoding audio file: {}", audio_path.display());
                }
            }
            Err(e) => {
                error!("Failed to open audio file {}: {}", audio_path.display(), e);
            }
        }

        player.sleep_until_end();
    });
}

fn main() -> Result<(), slint::PlatformError> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn"),
    )
    .init();

    info!("Starting schisaciaccole");

    let main_window = MainWindow::new().unwrap();
    let start_with_zero_millis_window = StartWithZeroMillis::new().unwrap();

    let (opts_time, opts_text) = compute_options();
    let slint_model = Rc::new(VecModel::from(opts_text));

    main_window.set_time_options(ModelRc::from(slint_model.clone()));

    let app_state = Rc::new(RefCell::new(AppState{
        opts_time,
        slint_model,
    }));

    main_window.on_select_time({
        let window_weak = main_window.as_weak();
        let state_clone = app_state.clone();

        move |index| {
            let window = window_weak.unwrap();
            let idx: usize = index.try_into().unwrap();
            let state = state_clone.borrow();
            let now: DateTime<Local> = Local::now();
            let target_time = state.opts_time[idx];
            let remaining_time = target_time - now;

            let rem_seconds = if remaining_time.num_seconds() > 0 {
                remaining_time.num_seconds()
            } else {
                Duration::minutes(15).num_seconds() + remaining_time.num_seconds()
            };

            info!(
                "select_time: idx={} target={} remaining={}s",
                idx,
                target_time.format("%H:%M"),
                rem_seconds
            );

            window.set_timer_time(rem_seconds * 1000);
            window.set_page(Page::TimerPage);
            window.set_running_state(true);
        }
    });

    // start/pause
    main_window.on_start_pause({
        let window_weak = main_window.as_weak();

        move || {
            let window = window_weak.unwrap();
            let now_running = !window.get_running_state();
            let timer_time = window.get_timer_time();
            if timer_time > 0 {
                info!("start_pause: running={} timer_time={}ms", now_running, timer_time);
                window.set_running_state(now_running);
            } else {
                info!("start_pause: blocked, timer_time is 0");
                let _ = start_with_zero_millis_window.run();
            }
        }
    });

    main_window.on_stop({
        let window_weak = main_window.as_weak();

        move || {
            let window = window_weak.unwrap();
            info!("stop: timer stopped, back to selection");

            window.set_running_state(false);
            window.set_timer_time(0);
            window.set_page(Page::SelectionPage);
        }
    });

    main_window.on_compute_options({
        let state_clone = app_state.clone();

        move || {
            let mut state = state_clone.borrow_mut();

            let (new_times, new_texts) = compute_options();

            debug!("compute_options refreshed {} options", new_texts.len());
            state.opts_time = new_times;
            state.slint_model.set_vec(new_texts);
        }
    });

    main_window.on_play_sound({
        || {
            play_sound();
        }
    });


    let name = env!("CARGO_PKG_NAME");
    let _ = i_slint_core::api::set_xdg_app_id(name);
    main_window.run()
}
