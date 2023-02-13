use std::sync::{Arc, Mutex};

use clap::Parser;
use eframe::egui::plot::{Line, Plot, PlotPoint, PlotPoints, PlotPoints::Owned};
use eframe::egui::{
    Align, CentralPanel, Color32, Context, FontFamily, Layout, SidePanel,
};
use eframe::{egui, App, Frame};

use crate::egui::RichText;

//https://github.com/emilk/egui/issues/2307 AUTO BOUNDS NOT WORKING EGUI IS BROKEN

#[derive(Debug, Clone)]
pub struct Dataset {
    capital: Vec<PlotPoint>,
    eur: Vec<PlotPoint>,
    usd: Vec<PlotPoint>,
    yen: Vec<PlotPoint>,
    yuan: Vec<PlotPoint>,
}

impl Dataset {
    fn new() -> Self {
        Dataset {
            capital: Vec::new(),
            eur: Vec::new(),
            usd: Vec::new(),
            yen: Vec::new(),
            yuan: Vec::new(),
        }
    }

    pub fn append_points(&mut self, message: Vec<&str>, count: f64) {
        let eur = message[0].parse::<f64>().unwrap();
        let usd = message[1].parse::<f64>().unwrap();
        let yen = message[2].parse::<f64>().unwrap();
        let yuan = message[3].parse::<f64>().unwrap();
        self.capital.push(PlotPoint {
            x: count,
            y: eur + usd + yen + yuan,
        });
        self.eur.push(PlotPoint { x: count, y: eur });
        self.usd.push(PlotPoint { x: count, y: usd });
        self.yen.push(PlotPoint { x: count, y: yen });
        self.yuan.push(PlotPoint { x: count, y: yuan });
    }

    pub fn get_points_conditional(&self, state: &str) -> PlotPoints {
        Owned(match state {
            "CAPITAL" => Vec::from_iter(self.capital.iter().cloned()),
            "EUR" => Vec::from_iter(self.eur.iter().cloned()),
            "USD" => Vec::from_iter(self.usd.iter().cloned()),
            "YEN" => Vec::from_iter(self.yen.iter().cloned()),
            "YUAN" => Vec::from_iter(self.yuan.iter().cloned()),
            _ => panic!("Invalid state"),
        })
    }
}

pub struct Visualizer {
    pub dataset_dropship: Arc<Mutex<Dataset>>,
    pub dataset_3m: Arc<Mutex<Dataset>>,
    state: String,
    widget1: bool,
    widget2: bool,
}

impl Visualizer {
    pub fn new() -> Self {
        Visualizer {
            dataset_dropship: Arc::new(Mutex::new(Dataset::new())),
            dataset_3m: Arc::new(Mutex::new(Dataset::new())),
            state: "CAPITAL".to_string(),
            widget1: true,
            widget2: true,
        }
    }
}

impl App for Visualizer {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        let args = crate::Args::parse();
        SidePanel::right("Graph")
            .show_separator_line(false)
            .exact_width(1000.0)
            .show(ctx, |ui| {
                let plot = Plot::new("cooltrader").auto_bounds_y();
                //getting which vector to show
                let data_dropship = self
                    .dataset_dropship
                    .lock()
                    .unwrap()
                    .get_points_conditional(self.state.as_str());
                let data_3m = self
                    .dataset_3m
                    .lock()
                    .unwrap()
                    .get_points_conditional(self.state.as_str());
                plot.show(ui, |plot_ui| {
                    if self.widget1 {
                        plot_ui.line(Line::new(data_dropship).width(5.0));
                    }
                    if self.widget2 {
                        plot_ui.line(
                            Line::new(data_3m)
                                .width(5.0)
                                .color(Color32::from_rgb(252, 15, 192)),
                        );
                    }
                });
            });
        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui_widget| {
                ui_widget.label("Trader1");
                ui_widget.add(toggle(&mut self.widget1));
                ui_widget.add_space(10.0);
                ui_widget.label("Trader2");
                ui_widget.add(toggle(&mut self.widget2));
            });
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui_centered| {
                ui_centered.separator();
                if ui_centered
                    .radio(
                        self.state == "CAPITAL".to_string(),
                        "CAPITAL",
                    )
                    .clicked()
                {
                    self.state = "CAPITAL".to_string();
                };
                if ui_centered
                    .radio(
                        self.state == "EUR".to_string(),
                        "EUR",
                    )
                    .clicked()
                {
                    self.state = "EUR".to_string();
                };
                if ui_centered
                    .radio(
                        self.state == "USD".to_string(),
                        "USD",
                    )
                    .clicked()
                {
                    self.state = "USD".to_string();
                };
                if ui_centered
                    .radio(
                        self.state == "YEN".to_string(),
                        "YEN",
                    )
                    .clicked()
                {
                    self.state = "YEN".to_string();
                };
                if ui_centered
                    .radio(
                        self.state == "YUAN".to_string(),
                        "YUAN",
                    )
                    .clicked()
                {
                    self.state = "YUAN".to_string();
                };
            });
            ui.with_layout(Layout::bottom_up(Align::Center), |ui_bottom| {
                ui_bottom.hyperlink_to(
                    format!("{} ZSE's GitHub", egui::special_emojis::GITHUB),
                    "https://github.com/StefanoDalMas/ZSE",
                );
                ui_bottom.separator();
                ui_bottom.label(
                    RichText::new(format!("Hello {}!", args.name))
                        .family(FontFamily::Monospace)
                        .size(15.0),
                );
                ui_bottom.separator();
            })
        });
        ctx.request_repaint();
    }
}

//debug functions
fn print_point(point: &PlotPoint) {
    println!("x: {}, y: {}", point.x, point.y);
}

fn print_vector(vector: &PlotPoints) {
    vector.points().iter().for_each( print_point);
}

//Custom Widget implementation
pub fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed(); // report back that the value changed
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));
    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }
    response
}
pub fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| toggle_ui(ui, on)
}

