extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let event_subsystem = sdl_context.event().unwrap();
    event_subsystem
        .register_custom_event::<PushGameplayScreen>()
        .expect("Already registered that event");

    event_subsystem
        .register_custom_event::<PopCurrentScreen>()
        .expect("Already registered that event");

    let window = video_subsystem
        .window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    let mut screens: ScreenStack = ScreenStack::new(event_subsystem);
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {
                    screens.process_event(&event);
                }
            }
        }
        // The rest of the game loop goes here...

        screens.run_tick();
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

struct PushGameplayScreen {}
struct PopCurrentScreen {}

trait Screen {
    fn run_tick(&mut self) -> bool;
    fn process_event(&mut self, event: &sdl2::event::Event) -> bool;
}

struct TitleScreen {
    event_sender: sdl2::event::EventSender,
}

impl Screen for TitleScreen {
    fn run_tick(&mut self) -> bool {
        println!("Title Screen!");
        false
    }

    fn process_event(&mut self, event: &sdl2::event::Event) -> bool {
        if let Event::KeyDown {
            keycode: Some(sdl2::keyboard::Keycode::F),
            ..
        } = event
        {
            self.event_sender
                .push_custom_event(PushGameplayScreen {})
                .expect("PushGameplayScreen not registered as an event.");
            true
        } else {
            false
        }
    }
}

impl TitleScreen {
    fn new(event_sender: sdl2::event::EventSender) -> Self {
        Self { event_sender }
    }
}

struct GameplayScreen {
    event_sender: sdl2::event::EventSender,
}

impl Screen for GameplayScreen {
    fn run_tick(&mut self) -> bool {
        println!("Gameplay Screen!");
        false
    }

    fn process_event(&mut self, event: &sdl2::event::Event) -> bool {
        if let Event::KeyDown {
            keycode: Some(sdl2::keyboard::Keycode::F),
            ..
        } = event
        {
            self.event_sender
                .push_custom_event(PopCurrentScreen {})
                .expect("PopCurrentScreen not registered as an event.");
            true
        } else {
            false
        }
    }
}

impl GameplayScreen {
    fn new(event_sender: sdl2::event::EventSender) -> Self {
        Self { event_sender }
    }
}

struct PauseScreen {}

impl Screen for PauseScreen {
    fn run_tick(&mut self) -> bool {
        println!("Pause Screen!");
        false
    }

    fn process_event(&mut self, _event: &sdl2::event::Event) -> bool {
        false
    }
}

enum Screens {
    Title(TitleScreen),
    Gameplay(GameplayScreen),
    //Pause(PauseScreen),
}

impl Screens {
    fn process_event(&mut self, event: &sdl2::event::Event) -> bool {
        match self {
            Screens::Title(title) => title.process_event(event),
            Screens::Gameplay(gameplay) => gameplay.process_event(event),
            //  Screens::Pause(pause) => pause.process_event(event),
        }
    }

    fn run_tick(&mut self) -> bool {
        match self {
            Screens::Title(title) => title.run_tick(),
            Screens::Gameplay(gameplay) => gameplay.run_tick(),
            //  Screens::Pause(pause) => pause.run_tick(),
        }
    }
}

struct ScreenStack {
    screens: Vec<Screens>,
    event_subsystem: sdl2::EventSubsystem,
}

impl ScreenStack {
    fn new(event_subsystem: sdl2::EventSubsystem) -> Self {
        Self {
            screens: vec![Screens::Title(TitleScreen::new(
                event_subsystem.event_sender(),
            ))],
            event_subsystem,
        }
    }

    fn process_event(&mut self, event: &sdl2::event::Event) -> bool {
        let screen_did_process: bool = if let Some(screen) = self.screens.last_mut() {
            screen.process_event(event)
        } else {
            false
        };

        if !screen_did_process {
            match event {
                Event::User { .. } => {
                    if let Some(_) = event.as_user_event_type::<PushGameplayScreen>() {
                        self.screens.push(Screens::Gameplay(GameplayScreen::new(
                            self.event_subsystem.event_sender(),
                        )));
                    } else if let Some(_) = event.as_user_event_type::<PopCurrentScreen>() {
                        self.screens.pop();
                    }
                }
                _ => {}
            }
        }

        true
    }

    fn run_tick(&mut self) -> bool {
        if let Some(screen) = self.screens.last_mut() {
            screen.run_tick()
        } else {
            false
        }
    }
}
