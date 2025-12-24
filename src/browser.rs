use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};

/// The main application state for the Browser.
/// 
/// In a more complex app, this struct would hold data like URLs, 
/// history, or scroll positions.
pub struct Browser {}

impl Default for Browser {
    /// Provides the default state for the Browser.
    fn default() -> Self {
        Browser {}
    }
}

impl Browser {
    /// Configures the initial context and returns a new instance of [`Browser`].
    ///
    /// # Arguments
    /// * `cc` - The creation context, used to set the visual theme and access the GPU.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Enforce light mode so black shapes are visible against the background.
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        Self::default()
    }
}

impl eframe::App for Browser {
    /// Called by the framework every frame to draw the UI.
    ///
    /// This implementation uses the `painter` to manually render:
    /// 1. A rectangle outline.
    /// 2. A circular ellipse.
    /// 3. A centered text label.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // The painter allows direct 2D drawing onto the UI layer.
            let painter = ui.painter();

            // Draw a rectangle (Equivalent to Tkinter's canvas.create_rectangle)
            // Coordinates: top-left (10, 20), bottom-right (400, 300)
            let my_rect = Rect::from_min_max(Pos2::new(10.0, 20.0), Pos2::new(400.0, 300.0));
            painter.add(Shape::rect_stroke(
                my_rect,
                0.0, // Corner rounding (0.0 = sharp corners)
                Stroke::new(1.0, Color32::BLACK),
                StrokeKind::Middle
            ));

            // Draw an ellipse (Equivalent to Tkinter's canvas.create_oval)
            // Parameters: Center (125, 125), Radius (25x25), Stroke style
            painter.add(Shape::ellipse_stroke(
                Pos2::new(125.0, 125.0), 
                Vec2::new(25.0, 25.0), 
                Stroke::new(1.0, Color32::BLACK)
            ));

            // Draw text (Equivalent to Tkinter's canvas.create_text)
            // Parameters: Position, Alignment (Anchor), String, Font, Color
            painter.text(
                Pos2::new(200.0, 150.0), 
                Align2::CENTER_CENTER, 
                "Hello, World!", 
                FontId::proportional(20.0), 
                Color32::BLACK
            );
        });
    }
}