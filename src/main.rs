
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use eframe:: { 
    App, 
    Frame
};

mod widgets;
use crate::widgets::errorfield::ErrorField;
use crate::widgets::switch::Switch;

mod models;
use crate::models::variable::Variable;
use crate::models::decoder::Decoder;
use crate::models::encoder::Encoder;

const WINDOW_SIZE:  egui::Vec2 = egui::Vec2::new(640.0, 480.0);
const ACCENT_COLOR: egui::Color32 = egui::Color32::from_rgb(204, 136, 0); // HSL(40,100,40)

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Copy, Clone)]
enum InterfaceMode
{
    Dark,
    Light
}

#[derive(Default, PartialEq)]
enum StateTracker 
{
    Dragging, // While dragging is progress.
    Saving,   // While saving is in progress.
    #[default] Idle
}

// Extract only message from an Result error by adding a new trait to Result (go Rust!).
trait MessageOnly {
    fn as_message (&self) -> String;
}

impl <T,E> MessageOnly for Result<T,E> where E: ToString {
    fn as_message (&self) -> String {
        match self {
            Ok  (_) => String::new(),
            Err (m) => m.to_string()
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Bitcoder
{
    ui_size: f32,
    ui_mode: InterfaceMode,

    #[serde(skip)] variables: Vec<Variable>,
    #[serde(skip)] outcome: Variable,
    #[serde(skip)] templates: Vec<String>,
    #[serde(skip)] messages: Vec<String>,
    #[serde(skip)] error: String,
    #[serde(skip)] path: String,
    #[serde(skip)] state: StateTracker
}

impl Default for Bitcoder
{
    fn default() -> Self {
        Self {
            ui_size: 1.2,
            ui_mode: InterfaceMode::Dark,
            variables: Vec::new(),
            outcome: Variable::default(),
            templates: Vec::new(),
            messages: Vec::new(),
            error: String::new(),
            path: String::new(),
            state: StateTracker::Idle
        }
    }
}

impl Bitcoder
{
    fn new (context: &eframe::CreationContext<'_>) -> Self {
        let object = if let Some(ps) = context.storage { eframe::get_value(ps, eframe::APP_KEY).unwrap_or_default() } else { Bitcoder::default() };
        Self::set_fonts(&context.egui_ctx);
        Self::set_style(&context.egui_ctx, object.ui_mode);
        context.egui_ctx.set_zoom_factor(object.ui_size);
        object
    }

    // Static method, used in new.
    fn set_fonts (context: &egui::Context) {
        let textfont = "Sans Font";
        let iconfont = "Icons";
        let mut fonts = egui::FontDefinitions::default();
        fonts
            .font_data
            .insert(
                textfont.to_string(), 
                std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/AtkinsonHyperlegibleNext-Regular.ttf")))
            );
        fonts
            .font_data
            .insert(
                iconfont.to_string(), 
                std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/MaterialIconsOutlined-Regular.otf"))
                    .tweak(egui::FontTweak { 
                        scale: 1.1, 
                        ..Default::default() 
                    }
            )));
        if let Some(p) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            p.insert(0, textfont.to_string());
            p.insert(1, iconfont.to_string());
            context.set_fonts(fonts);
        }
    }

    // Static method, used in new.
    fn set_style (context: &egui::Context, mode: InterfaceMode) {
        let mut visuals: egui::Visuals;
        match mode {
            InterfaceMode::Dark  => {
                context.set_theme(egui::Theme::Dark);
                visuals = egui::Visuals::dark();
                visuals.override_text_color = Some(egui::Color32::WHITE);
            },
            InterfaceMode::Light => {
                context.set_theme(egui::Theme::Light);
                visuals = egui::Visuals::light();
                visuals.override_text_color = Some(egui::Color32::BLACK);
            }
        }
        visuals.widgets.active.bg_fill = ACCENT_COLOR;
        visuals.widgets.noninteractive.bg_fill = ACCENT_COLOR;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, ACCENT_COLOR.gamma_multiply(0.25));
        visuals.widgets.hovered.bg_fill = ACCENT_COLOR;
        visuals.selection.stroke.color  = ACCENT_COLOR; 
        visuals.selection.bg_fill = ACCENT_COLOR.gamma_multiply(0.35);
        visuals.slider_trailing_fill = true;
        context.style_mut(|style| {
            style.spacing.item_spacing = egui::Vec2::new(12.0, 8.0);
            style.spacing.button_padding = egui::Vec2::new(8.0, 2.0);
        });
        context.set_visuals(visuals);
    }
    
    fn get_main_frame (&mut self) -> egui::Frame {
        let color = match self.ui_mode {
            InterfaceMode::Dark  => egui::Color32::from_rgb( 20,  15,  10),
            InterfaceMode::Light => egui::Color32::from_rgb(250, 245, 240)
        };
        egui::Frame {
            inner_margin: egui::Margin::same(24),
            fill: color,
            ..Default::default()
        }
    }

    fn get_card_frame (&mut self) -> egui::Frame {
        self.get_main_frame()
            .inner_margin(18.0)
            .outer_margin(4.0)
            .corner_radius(12.0)
            .fill(ACCENT_COLOR.gamma_multiply(0.1))
            .stroke(egui::Stroke::new(2.0, ACCENT_COLOR.gamma_multiply(0.2)))
    }

    fn ui_outcome (&mut self, ui: &mut egui::Ui, context: &egui::Context) {
        ui.vertical(|ui| {
            // ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(egui::RichText::new("OUTCOME VARIABLE:").small().weak());
            ui.label(egui::RichText::new(self.outcome.name()).heading().color(ACCENT_COLOR));
            ui.label(format!("2953 variables will be created. Outcome ranges from {}", self.outcome.range()));
        });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let dragger = ui.button("\u{e074} Drag to export").interact(egui::Sense::click_and_drag()).highlight();
            if  dragger.drag_started() {
                self.state = StateTracker::Dragging;
            }
            let outside = !context.screen_rect().contains(ui.input(|i| i.pointer.interact_pos()).unwrap_or_default());
            if  dragger.drag_stopped() {
                context.set_cursor_icon(egui::CursorIcon::Default);
                self.state = StateTracker::Idle;
                if outside {
                    // thread::spawn(|| { // Should be this easy (nogo Rust).
                        self.save_file();
                    // });
                }
            }
            if self.state == StateTracker::Dragging {
                context.set_cursor_icon(if outside { egui::CursorIcon::Grabbing } else { egui::CursorIcon::NoDrop });
            }
        });
    }

    fn ui_settings (&mut self, ui: &mut egui::Ui, context: &egui::Context) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("TEXT SIZE").small().weak());
                if ui.add(egui::Slider::new(&mut self.ui_size, 1.0..=1.7)).changed() {
                    context.set_zoom_factor(self.ui_size);
                }
            });
            ui.add_space(24.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("DARK MODE").small().weak());
                if ui.add(Switch::new(InterfaceMode::Dark == self.ui_mode)).clicked() {
                    match self.ui_mode {
                        InterfaceMode::Dark  => { 
                            self.ui_mode = InterfaceMode::Light;
                            Self::set_style(ui.ctx(), InterfaceMode::Light);
                        },
                        InterfaceMode::Light => { 
                            self.ui_mode = InterfaceMode::Dark;
                            Self::set_style(ui.ctx(), InterfaceMode::Dark);
                        }
                    }
                }
            });
        });
    }

    fn ui_card (&mut self, ui: &mut egui::Ui, index: usize) {
        self.get_card_frame().show(ui, |ui| {
            let variable = &self.variables[index];
            ui.label(egui::RichText::new(variable.name()).heading());
            ui.separator();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let mut tmp = false;
                    ui.checkbox(&mut tmp, "Include this");
                    ui.checkbox(&mut tmp, "As numeric");
                });
                ui.add_space(24.0);
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    ui.radio(true, "Recode all unique values");
                    ui.radio(true, "Use expression below to categorize:");
                    if ui.add(ErrorField::new(&mut self.templates[index], self.messages[index].is_empty())).changed() {
                        self.messages[index] = String::from("TEST"); //TODO: Use Parser
                    }
                    if !self.messages[index].is_empty() {
                        ui.label(egui::RichText::new(&self.messages[index]).color(egui::Color32::RED));
                    }
                });
            });
            ui.separator();
            ui.label(format!("Results in 893 variables from 1014 values (215 missing). Ranges from {}.", variable.range()));
        });
    }

    fn ui_list (&mut self, ui: &mut egui::Ui) {
        let count = self.variables.len();
        if  count > 0 {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for index in 0..count {
                    self.ui_card(ui, index);
                };
            });
        }
    }

    fn load_file (&mut self) {
        self.error = Decoder::load(self.path.as_str(), &mut self.variables).as_message();
        self.templates = Vec::with_capacity(self.variables.len()); // One template string and one ...
        self.messages  = Vec::with_capacity(self.variables.len()); // error message per variable in the UI.
        self.variables.iter().for_each(|_| {
            self.templates.push(String::new());
            self.messages. push(String::new());
        });
        // Last variable is always the outcome variable.
        if let Some(variable) = self.variables.pop() {
            self.outcome = variable;
            self.outcome.as_numbers();
        }
    }

    fn save_file (&mut self) {
        self.state = StateTracker::Saving;
        self.error = Encoder::save(self.path.as_str(), &self.variables).as_message();
        self.state = StateTracker::Idle;
    }

}

impl App for Bitcoder
{
    fn save (&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update (&mut self, context: &egui::Context, _frame: &mut Frame) {
        egui::TopBottomPanel::bottom("Settings").frame(self.get_main_frame()).resizable(false).show(context, |ui| {
            self.ui_settings(ui, context);
        });
        if !self.variables.is_empty() {
            egui::TopBottomPanel::bottom("Variable").frame(self.get_main_frame()).resizable(false).show(context, |ui| {
                self.ui_outcome(ui, context);
            });
        }
        egui::CentralPanel::default().frame(self.get_main_frame()).show(context, |ui| {
            if !self.error.is_empty() {
                egui::Modal::new(egui::Id::new("Dialog")).frame(self.get_main_frame()).show(ui.ctx(), |ui| {
                    ui.set_width(200.0);
                    ui.style_mut().spacing.item_spacing = egui::Vec2::new(18.0, 12.0);
                    ui.label(egui::RichText::new("Oh no! An error has occured!").color(ACCENT_COLOR).weak());
                    ui.label(egui::RichText::new(&self.error).strong());
                    if ui.button("Ok").clicked() {
                        self.error.clear();
                    }
                });
            }
            let mut hovered = egui::HoveredFile::default();
            let mut dropped = egui::DroppedFile::default();
            context.input(|input| {
                if !input.raw.hovered_files.is_empty() { hovered = input.raw.hovered_files[0].clone() }
                if !input.raw.dropped_files.is_empty() { dropped = input.raw.dropped_files[0].clone() }
            });
            if hovered.path.is_some() {                
                ui.painter().rect(
                    ui.max_rect(), 
                    0.0, 
                    ui.style().visuals.selection.bg_fill, 
                    egui::Stroke::new(2.0, ACCENT_COLOR), 
                    egui::StrokeKind::Middle
                );
            }
            if dropped.path.is_some() {
                if let Some(path) = &dropped.path {
                    self.path = path.display().to_string();
                    self.load_file();
                }
            }
            if self.variables.is_empty() {
                ui.add_sized(ui.available_size(), egui::Label::new(egui::RichText::new("(drop file here)").heading().italics().weak()));
            } else {
                self.ui_list(ui);
            }
        });
    }

}

fn main() -> eframe::Result {
    eframe::run_native(
        "Bitcoder", 
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_resizable(true)
                .with_maximize_button(true)
                .with_minimize_button(true)
                .with_min_inner_size(WINDOW_SIZE)
                .with_inner_size(WINDOW_SIZE)
                .with_icon(eframe::icon_data::from_png_bytes(&include_bytes!("../assets/Bitcoder.png")[..]).unwrap_or_default()),
            ..Default::default()
        },
        Box::new(|context| {
            Ok(Box::new(Bitcoder::new(context)))
        })
    )
}
