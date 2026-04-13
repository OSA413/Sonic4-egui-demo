
use egui::{
    Align2,
    Color32,
    Context,
    FontData,
    FontDefinitions,
    FontFamily,
    FontId,
    FontTweak,
    Key,
    Modifiers,
    Pos2,
    Rect,
    RichText,
    ScrollArea,
    Slider,
    Stroke,
    StrokeKind,
    Widget,
    ImageSource,
};

use std::sync::Once;

// most of this code is ported over from sy1ntexx's d3d11 implementation.
pub fn ui(ctx: &Context, i: &mut i32) {
    unsafe {
        // You should not use statics like this, it's made
        // this way for the sake of example.
        static mut UI_CHECK: bool = true;
        static mut TEXT: Option<String> = None;
        static mut VALUE: f32 = 0.;
        static mut COLOR: [f32; 3] = [0., 0., 0.];
        static ONCE: Once = Once::new();

        ONCE.call_once(|| {
            let mut fonts = FontDefinitions::default();
            let tweak = FontTweak::default();
            fonts.font_data.insert(
                "Lobster-Regular".to_owned(),
                FontData::from_static(include_bytes!("../../../fonts/Lobster/Lobster-Regular.ttf")).tweak(tweak).into(),
            );
            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .push("Lobster-Regular".to_owned());
            fonts
                .families
                .get_mut(&FontFamily::Monospace)
                .unwrap()
                .push("Lobster-Regular".to_owned());
            ctx.set_fonts(fonts);
            egui_extras::install_image_loaders(ctx);
        });

        if TEXT.is_none() {
            TEXT = Some(String::from("Test"));
        }

        ctx.debug_painter().text(
            Pos2::new(0., 0.),
            Align2::LEFT_TOP,
            "Bruh",
            FontId::default(),
            Color32::RED,
        );

        egui::containers::Window::new("Main menu").show(ctx, |ui| {
            ctx.settings_ui(ui);
            ui.label(RichText::new("Test").color(Color32::BLACK));
            ui.label(RichText::new("Other").color(Color32::WHITE));
            ui.separator();

            ui.label(RichText::new(format!("I: {}", *i)).color(Color32::LIGHT_RED));

            let input = ctx.input(|input| input.pointer.clone());
            ui.label(format!(
                "X1: {} X2: {}",
                input.button_down(egui::PointerButton::Extra1),
                input.button_down(egui::PointerButton::Extra2)
            ));

            let mods = ui.input(|input| input.modifiers);
            ui.label(format!(
                "Ctrl: {} Shift: {} Alt: {}",
                mods.ctrl, mods.shift, mods.alt
            ));

            if ui.input(|input| {
                input.modifiers.matches(Modifiers::CTRL) && input.key_pressed(Key::R)
            }) {
                println!("Pressed");
            }

            ui.checkbox(&mut UI_CHECK, "Some checkbox");
            ui.text_edit_singleline(TEXT.as_mut().unwrap());
            ScrollArea::vertical().max_height(200.).show(ui, |ui| {
                for i in 1..=100 {
                    ui.label(format!("Label: {}", i));
                }
            });

            Slider::new(&mut VALUE, -1.0..=1.0).ui(ui);

            ui.color_edit_button_rgb(&mut COLOR);
        

            ui.label(format!(
                "{:?}",
                &ui.input(|input| input.pointer.button_down(egui::PointerButton::Primary))
            ));
            if ui.button("You can't click me yet").clicked() {
                *i += 1;
            }
        });

        egui::Window::new("Image").show(ctx, |ui| {
            const IMG: ImageSource = egui::include_image!("../../../images/500px-Vulpes_vulpes_ssp_fulvus.webp");
            ui.image(IMG);
        });

        egui::Window::new("xd").show(ctx, |ui| {
            ctx.memory_ui(ui);
        });

        egui::Window::new("stuff").show(ctx, |ui| {
            ctx.inspection_ui(ui);
        });

        ctx.debug_painter().rect(
            Rect {
                min: Pos2::new(200.0, 200.0),
                max: Pos2::new(250.0, 250.0),
            },
            10.0,
            Color32::from_rgba_premultiplied(255, 0, 0, 150),
            Stroke::NONE,
            StrokeKind::Inside,
        );

        // this is supposed to be color channel testing to identify if any channels have been misplaced
        ctx.debug_painter().circle(
            Pos2::new(350.0, 350.0),
            35.0,
            Color32::from_rgba_premultiplied(255, 0, 0, 0),
            Stroke::NONE,
        );

        ctx.debug_painter().circle(
            Pos2::new(450.0, 350.0),
            35.0,
            Color32::from_rgba_premultiplied(0, 255, 0, 0),
            Stroke::NONE,
        );

        ctx.debug_painter().circle(
            Pos2::new(550.0, 350.0),
            35.0,
            Color32::from_rgba_premultiplied(0, 0, 255, 0),
            Stroke::NONE,
        );

        ctx.debug_painter().circle(
            Pos2::new(650.0, 350.0),
            35.0,
            Color32::from_rgba_premultiplied(0, 0, 0, 255),
            Stroke::new(5f32, Color32::from_rgba_premultiplied(0, 0, 255, 255)),
        );
    }
}