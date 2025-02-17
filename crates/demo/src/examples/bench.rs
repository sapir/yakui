use yakui::Color3;

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    let colors = [Color3::RED, Color3::GREEN, Color3::BLUE];

    yakui::row(|| {
        for x in 0..100 {
            yakui::column(|| {
                for y in 0..100 {
                    let color = colors[(x + y) % colors.len()];

                    let w = 2.0 + 3.0 + (x / 2) as f32;
                    let h = 2.0 + 3.0 + (y / 2) as f32;
                    yakui::colored_box(color, [w, h]);
                }
            });
        }
    });
}
