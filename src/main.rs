slint::include_modules!();


fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new().unwrap();
    let start_with_zero_millis_window = StartWithZeroMillis::new().unwrap();

    // it runs on the ui thread
    // let timer = slint::Timer::default();

    // start/pause
    main_window.on_start_pause({
        let window_weak = main_window.as_weak();

	move || {
	    let window = window_weak.unwrap();
            let now_running = !window.get_running_state();
            let milliseconds_elapsed = window.get_milliseconds_elapsed();
            if milliseconds_elapsed > 0 { 
                window.set_running_state(now_running);
            } else {
                let _ = start_with_zero_millis_window.run();
            }
	}
    });

    main_window.on_reset({
        let window_weak = main_window.as_weak();

        move || {
            let window = window_weak.unwrap();

            window.set_running_state(false);
            window.set_milliseconds_elapsed(0);
        }
    });

    main_window.run()
}
