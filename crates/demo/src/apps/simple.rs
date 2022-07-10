use yakui::{Button, Color3, Padding};

use crate::AppState;

pub fn app(_state: &AppState) {
    yakui::column(|| {
        let res = yakui::button(Button::styled([70.0, 30.0]));
        if res.clicked {
            println!("Clicked the first button!");
        }

        yakui::colored_box(Color3::REBECCA_PURPLE, || {
            let padding = Padding::even(8.0);
            yakui::pad(padding, || {
                yakui::row(|| {
                    yakui::button(Button::styled([80.0, 30.0]));
                    yakui::button(Button::styled([40.0, 30.0]));
                    yakui::button(Button::styled([60.0, 30.0]));
                });
            });
        });

        yakui::button(Button::styled([90.0, 30.0]));
    });
}