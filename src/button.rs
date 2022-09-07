use core::time::Duration;
use embedded_hal::digital::blocking::InputPin;
use std::time::Instant;

type Callback<I> = Box<dyn Fn(&I)>;

pub struct Button<I>
where
    I: InputPin,
{
    pin: I,
    button_state: ButtonState,
    press_time: Instant,
    when_pressed: WhenPressed,
    long_press_duration: Duration,
    short_action: Callback<I>,
    long_action: Callback<I>,
}

pub enum ButtonEvent {
    ShortPress,
    LongPress,
}

#[derive(Default)]
pub enum ButtonState {
    #[default]
    Released,
    Pressed,
}

#[derive(Default)]
pub enum WhenPressed {
    High,
    #[default]
    Low,
}

impl<I> Button<I>
where
    I: InputPin,
{
    pub fn new(pin: I) -> Self {
        let when_pressed = WhenPressed::Low;
        let button_state = ButtonState::Released;
        let press_time = Instant::now();
        let long_press_duration = Duration::from_millis(500);

        Self {
            pin,
            button_state,
            press_time,
            when_pressed,
            long_press_duration,
            short_action: Box::new(|_| {}),
            long_action: Box::new(|_| {}),
        }
    }

    pub fn set_high(mut self) -> Self {
        self.when_pressed = WhenPressed::High;
        self
    }

    pub fn set_low(mut self) -> Self {
        self.when_pressed = WhenPressed::Low;
        self
    }

    pub fn set_long_press_duration(mut self, duration: Duration) -> Self {
        self.long_press_duration = duration;
        self
    }

    pub fn set_short_action(&mut self, f: Callback<I>) -> &mut Self {
        self.short_action = f;
        self
    }

    pub fn set_long_action(&mut self, f: Callback<I>) -> &mut Self {
        self.long_action = f;
        self
    }

    pub fn is_pressed(&mut self) -> bool {
        match self.when_pressed {
            WhenPressed::High => self.pin.is_high().unwrap(),
            WhenPressed::Low => self.pin.is_low().unwrap(),
        }
    }

    pub fn poll(&mut self) {
        let event: Option<ButtonEvent> = match self.button_state {
            ButtonState::Released => {
                if self.is_pressed() {
                    self.button_state = ButtonState::Pressed;
                    self.press_time = Instant::now();
                }
                None
            }
            ButtonState::Pressed => {
                if !self.is_pressed() {
                    self.button_state = ButtonState::Released;
                    let press_duration = Instant::now().duration_since(self.press_time).as_millis();

                    if press_duration < self.long_press_duration.as_millis() {
                        Some(ButtonEvent::ShortPress)
                    } else {
                        Some(ButtonEvent::LongPress)
                    }
                } else {
                    None
                }
            }
        };

        match event {
            Some(ButtonEvent::ShortPress) => {
                (self.short_action)(&self.pin);
            }
            Some(ButtonEvent::LongPress) => {
                (self.long_action)(&self.pin);
            }
            _ => {}
        };
    }
}
