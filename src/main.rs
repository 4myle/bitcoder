
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui::{self, Color32};
use eframe:: { 
    App, 
    Frame
};

mod widgets;
use widgets::errorfield::ErrorField;
use widgets::switch::Switch;

mod models;
use models::variable::Variable;
use models::variable::Mapping;
use models::decoder::Decoder;
use models::encoder::Encoder;
use models::parser::Parser;

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

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct Card 
{
    expression: String, // Expression to cluster values (if in Cluster mode).
    is_included: bool,  // If variable is included (GUI)
    is_numeric: bool,   // If variable should be perceived as having string or numeric values.
    title: String,      // Title of variable that can be edited.
    #[serde(skip)]
    message: String,    // Message after parsing expression.
}

impl Card 
{
    fn new (name: &str) -> Self {
        Self {
            is_included: true,
            title: name.to_string(),
            ..Default::default()
        }
    }
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

    // #[serde(skip)] storage: dyn eframe::Storage,
    #[serde(skip)] variables: Vec<Variable>,
    #[serde(skip)] outcome: Variable,
    #[serde(skip)] cards: Vec<Card>,
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
            cards: Vec::new(),
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
                std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/Inter-Regular.ttf")))
            );
        fonts
            .font_data
            .insert(
                iconfont.to_string(), 
                std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/MaterialIconsOutlined-Regular.otf"))
                    .tweak(egui::FontTweak { 
                        scale: 1.2, 
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
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(2.0, ACCENT_COLOR);
        visuals.widgets.inactive.bg_fill = ACCENT_COLOR.gamma_multiply(0.20);
        visuals.widgets.active.bg_fill = ACCENT_COLOR;
        visuals.widgets.noninteractive.bg_fill = ACCENT_COLOR;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, ACCENT_COLOR.gamma_multiply(0.25));
        visuals.widgets.hovered.bg_fill = ACCENT_COLOR;
        visuals.selection.stroke.color  = ACCENT_COLOR; 
        visuals.selection.bg_fill = ACCENT_COLOR.gamma_multiply(0.30);
        // visuals.slider_trailing_fill = true;
        context.style_mut(|style| {
            style.spacing.item_spacing = egui::Vec2::new(12.0, 8.0);
            style.spacing.button_padding = egui::Vec2::new(8.0, 4.0);
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

    fn get_card_frame (&mut self, index: usize) -> egui::Frame {
        let color = if self.cards[index].is_included {ACCENT_COLOR} else {Color32::GRAY};
        self.get_main_frame()
            .inner_margin(18.0)
            .outer_margin(4.0)
            .corner_radius(12.0)
            .fill(color.gamma_multiply(0.1))
            .stroke(egui::Stroke::new(2.0, color.gamma_multiply(0.2)))
    }

    fn get_over_frame (&mut self) -> egui::Frame {
        self.get_main_frame()
            .inner_margin(18.0)
            .outer_margin(4.0)
            .corner_radius(12.0)
            .shadow(egui::Shadow { offset: [4,4], blur: 8, spread: 0, color: Color32::BLACK.gamma_multiply(0.3) })
            .stroke(egui::Stroke::new(2.0, ACCENT_COLOR.gamma_multiply(0.2)))
    }

    fn ui_card (&mut self, ui: &mut egui::Ui, index: usize) {
        self.get_card_frame(index).show(ui, |ui| {
            let variable = &mut self.variables[index];
            let card = &mut self.cards[index];
            ui.horizontal(|ui| {
                ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
                ui.style_mut().visuals.extreme_bg_color = Color32::TRANSPARENT;
                ui.label(if card.is_numeric {"\u{e9ef}"} else {"\u{eb94}"});
                if ui.text_edit_singleline(&mut card.title).changed() {
                    if  card.title.is_empty() {
                        card.title = variable.name().to_string();
                        self.error = String::from("Name of variable can not be empty.");
                    } else {
                        variable.set_name(card.title.as_str());
                    }
                }
            });
            ui.separator();
            if  !card.is_included {
                ui.checkbox(&mut card.is_included, "Include this");
                if card.is_included {
                    variable.include();
                } else {
                    variable.exclude();
                }
                return;
            }
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let card = &mut self.cards[index];
                    if ui.checkbox(&mut card.is_included, "Include this").changed() && card.is_included {
                        variable.include();
                    }
                    if ui.checkbox(&mut card.is_numeric, "As numeric").changed() {
                        if card.is_numeric {
                            variable.as_numbers();
                        } else {
                            variable.as_strings();
                        }
                    }
                });
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    let (is_recoded, is_cluster) = match variable.mapping() {
                        Mapping::Recode => (true, false),
                        Mapping::Cluster {..} => (false, true)
                    };
                    if ui.radio(is_recoded, "Recode all unique values").clicked() {
                        variable.set_recoded();
                    }
                    if ui.radio(is_cluster, "Use expression to create clusters").clicked() {
                        variable.set_cluster();
                    }
                    if is_cluster {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                            let card = &mut self.cards[index];
                            if ui.add(ErrorField::new(&mut card.expression, card.message.is_empty())).changed() {
                                match Parser::parse(&card.expression) {
                                    Err(m) => card.message = m,
                                    Ok (t) => {
                                        match variable.use_ranges(&t) {
                                            Ok (()) => card.message.clear(),
                                            Err(m)  => card.message = m.to_string()
                                        }
                                    }
                                }
                            }
                            if !card.message.is_empty() {
                                ui.label(egui::RichText::new(&self.cards[index].message).color(egui::Color32::RED));
                            }
                        });
                    }
                    // Add boxplot? https://github.com/emilk/egui_plot and https://github.com/emilk/egui_plot/issues/9
                });
            });
            ui.separator();
            ui.label(format!("Results in {} bit variables ({} missing). Ranges from {} to {}", 
                variable.density().len(), // = number of clusters or number of unique values if recoded.
                variable.missing(), 
                variable.minimum(), 
                variable.maximum())
            );
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

    fn ui_outcome (&mut self, ui: &mut egui::Ui, context: &egui::Context) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("OUTCOME VARIABLE:").small().weak());
            ui.label(egui::RichText::new(self.outcome.name()).heading().color(ACCENT_COLOR));
            ui.label(format!("{} bit variables in total will be created. Outcome ranges from {} to {}", 
                self.variables.iter().map(|v| v.density().len()).reduce(|a, v| a + v).unwrap_or(0),
                self.outcome.minimum(), 
                self.outcome.maximum())
            );
        });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let dragger = ui.button("\u{e945} Drag to export").interact(egui::Sense::click_and_drag()).highlight();
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

    fn load_file (&mut self, storage: &dyn eframe::Storage) {
        self.error = Decoder::load(self.path.as_str(), &mut self.variables).as_message();
        self.cards = Vec::with_capacity(self.variables.len());
        // Last variable is the outcome variable (interpretable as an f32).
        if let Some(variable) = self.variables.pop() {
            self.outcome = variable;
            self.outcome.as_numbers();
        }
        if let Some(stem) = std::path::PathBuf::from(&self.path).file_stem() {
            if let Some(name) = stem.to_str() {
                self.cards = eframe::get_value(storage, name).unwrap_or_default();
                // Recalculate variables marked as numeric and set cluster expression, if any.
                self.cards.iter_mut().enumerate().for_each(|c| {
                    if c.1.is_numeric {
                        self.variables[c.0].as_numbers();
                    }
                    if !c.1.expression.is_empty() {
                        match Parser::parse(&c.1.expression) {
                            Err(m) => c.1.message = m,
                            Ok (t) => {
                                match self.variables[c.0].use_ranges(&t) {
                                    Ok (()) => c.1.message.clear(),
                                    Err(m)  => c.1.message = m.to_string()
                                }
                            }
                        }
                    }
                });
            }
        }
        // Fill rest of cards collection, or all if none was deserialized.
        for index in self.cards.len()..self.variables.len() {
            self.cards.push(Card::new(self.variables[index].name()));
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
        if !self.path.is_empty() {
            if let Some(stem) = std::path::PathBuf::from(&self.path).file_stem() {
                if let Some(name) = stem.to_str() {
                    eframe::set_value(storage, name, &self.cards);
                }
            }
        }
    }

    fn update (&mut self, context: &egui::Context, frame: &mut Frame) {
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
                egui::Modal::new(egui::Id::new("Dialog")).frame(self.get_over_frame()).show(ui.ctx(), |ui| {
                    ui.set_width(240.0);
                    ui.style_mut().spacing.item_spacing = egui::Vec2::new(18.0, 12.0);
                    ui.label(egui::RichText::new("Oh no! An error occured.").color(ACCENT_COLOR).weak());
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
                    self.load_file(Option::unwrap(frame.storage()));
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
            let application = Box::new(Bitcoder::new(context));
            // *application.storage = context.storage;
            Ok (application)
        })
    )
}
