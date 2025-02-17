use yakui::widgets::Pad;

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    yakui::column(|| {
        let res = yakui::button("First button");
        if res.clicked {
            println!("Clicked the first button!");
        }

        let padding = Pad::all(8.0);
        yakui::pad(padding, || {
            yakui::row(|| {
                yakui::button("Hello");
                yakui::button("World");
                yakui::button("I'm Yakui!");
            });
        });

        yakui::button("Sincerely, Yakui");
    });
}
