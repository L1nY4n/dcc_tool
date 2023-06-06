use crate::model::Dcc;
use crate::model::DccCmd;
use std::collections::HashMap;
use std::thread;

use eframe::{
    egui::{
        self, menu,
        plot::{Bar, BarChart, Plot},
        Button, CentralPanel, CollapsingHeader, Context, CursorIcon, Frame, Grid, Label, Layout,
        RichText, Separator, SidePanel, Slider, Style, TextStyle, TopBottomPanel,
    },
    emath::Align,
    epaint::{Color32, Vec2},
    App, CreationContext,
};
use serde::{Deserialize, Serialize};

use super::backend::{
    message::{ToBackend, ToFrontend},
    Backend,
};
use crossbeam::channel::{Receiver, Sender};

#[derive(Default)]
pub struct DccTestApp {
    setting: Setting,
    dccs: HashMap<u64, Dcc>,
    // Data transferring
    front_tx: Option<Sender<ToBackend>>,
    back_rx: Option<Receiver<ToFrontend>>,
}

#[derive(Default, Serialize, Deserialize)]
struct Setting {
    open: bool,
    show_color: bool,
    interval: u32,
    stacks: String,
    adding_code: String,
}

// fn setup_custom_fonts(ctx: &egui::Context) {
//     let mut fonts = egui::FontDefinitions::default();
//     fonts.font_data.insert(
//         "my_font".to_owned(),
//         egui::FontData::from_static(include_bytes!(
//             "../../fonts/genShiGoThic/GenShinGothic-Medium.ttf"
//         )),
//     );

//     fonts
//         .families
//         .entry(egui::FontFamily::Monospace)
//         .or_default()
//         .insert(0, "my_font".to_owned());

//     fonts
//         .families
//         .entry(egui::FontFamily::Proportional)
//         .or_default()
//         .push("my_font".to_owned());

//     ctx.set_fonts(fonts);
// }

impl DccTestApp {
    pub fn new(cc: &CreationContext) -> Self {
        let mut new_app = Self::default();
        new_app.configure_style(&cc.egui_ctx);
        //setup_custom_fonts(&cc.egui_ctx);
        let (front_tx, front_rx) = crossbeam::channel::unbounded();
        let (back_tx, back_rx) = crossbeam::channel::unbounded();

        if let Some(storage) = cc.storage {
            if let Some(setting) = eframe::get_value(storage, eframe::APP_KEY) {
                new_app.setting = setting
            }
        }
        let codes = new_app.setting.stacks.clone();
        thread::spawn(|| Backend::new(back_tx, front_rx, codes).init());
        new_app.front_tx = Some(front_tx);
        new_app.back_rx = Some(back_rx);
        new_app
    }

    fn configure_style(&self, ctx: &Context) {
        let style = Style { ..Style::default() };
        ctx.set_style(style);
    }

    fn render_top_panel(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // define a TopBottomPanel widget
        let f = Frame::none();
        TopBottomPanel::top("top_panel").frame(f).show(ctx, |ui| {
            ui.add_space(2.0);
            menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.add_space(5.0);
                    let rocket_btn = ui
                        .add(Button::new(
                            RichText::new("💰")
                                .text_style(egui::TextStyle::Heading)
                                .color(Color32::YELLOW),
                        ))
                        .on_hover_cursor(CursorIcon::Move);
                    if rocket_btn.is_pointer_button_down_on() {
                        frame.drag_window()
                    };
                });
                // controls
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let close_btn = ui.add(Button::new(
                        RichText::new("❌")
                            .text_style(TextStyle::Body)
                            .color(Color32::RED),
                    ));
                    if close_btn.clicked() {
                        frame.close();
                    }

                    let refresh_btn = ui.add(Button::new(
                        RichText::new("🔄")
                            .text_style(TextStyle::Body)
                            .color(Color32::GREEN),
                    ));
                    if refresh_btn.clicked() {}

                    // config button
                    let config_btn = ui.add(Button::new(
                        RichText::new("🛠")
                            .text_style(egui::TextStyle::Body)
                            .color(Color32::LIGHT_BLUE),
                    ));

                    if config_btn.clicked() {
                        self.setting.open = !self.setting.open
                    }
                });
            });
            ui.add_space(2.0);
            ui.add(Separator::default().spacing(0.0));
        });
    }

    // fn render_stocks(&self, ui: &mut eframe::egui::Ui) {
    //     use egui_extras::{Size, TableBuilder};
    //     ui.add_space(2.0);
    //     TableBuilder::new(ui)
    //         .striped(true)
    //         .cell_layout(egui::Layout::left_to_right(Align::Center))
    //         .column(Size::initial(60.0))
    //         .column(Size::initial(40.0).at_least(20.0))
    //         .column(Size::remainder().at_least(30.0))
    //         .resizable(false)
    //         .body(|mut body| {
    //             for s in &self.stocks {
    //                 body.row(18.0, |mut row| {
    //                     row.col(|ui| {
    //                         ui.centered_and_justified(|ui| {
    //                             ui.add(Label::new(
    //                                 RichText::new(s.name.to_string())
    //                                     .text_style(egui::TextStyle::Body),
    //                             ));
    //                         });
    //                     });
    //                     row.col(|ui| {
    //                         ui.centered_and_justified(|ui| {
    //                             ui.add(Label::new(
    //                                 RichText::new(s.new.to_string())
    //                                     .text_style(egui::TextStyle::Body),
    //                             ));
    //                         });
    //                     });
    //                     row.col(|ui| {
    //                         let color = if self.setting.show_color {
    //                             match s.rise_per {
    //                                 p if p < 0.0 => Color32::GREEN,
    //                                 n if n > 0.0 => Color32::RED,
    //                                 _ => Color32::WHITE,
    //                             }
    //                         } else {
    //                             Color32::WHITE
    //                         };
    //                         //   ui.centered_and_justified(|ui| {
    //                         ui.add(Label::new(
    //                             RichText::new(s.rise_per.to_string())
    //                                 .text_style(egui::TextStyle::Body)
    //                                 .color(color),
    //                         ));
    //                         //   });
    //                     });
    //                 })
    //             }
    //         });
    // }

    fn render_dccs(&mut self, ctx: &Context, ui: &mut eframe::egui::Ui) {
        for (dev_id, dcc) in self.dccs.iter_mut() {
            egui::Window::new(format!("{}[{:X}]", dcc.addr, dev_id))
                .resizable(true)
                .show(ctx, |ui| {
               
                        ui.add_space(2.0);
                        menu::bar(ui, |ui| {
                            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                                ui.add_space(5.0);
                                let _ = ui
                                    .add(Button::new(
                                        RichText::new("📛")
                                            .text_style(egui::TextStyle::Heading)
                                            .color(Color32::YELLOW),
                                    ));
                                  
                            });
                            // controls
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let reboot = ui.add(Button::new(
                                    RichText::new("📴")
                                        .text_style(TextStyle::Body)
                                        .color(Color32::RED),
                                ));
                                if reboot.clicked() {
                                  
                                }
            
                                // let refresh_btn = ui.add(Button::new(
                                //     RichText::new("🔄")
                                //         .text_style(TextStyle::Body)
                                //         .color(Color32::GREEN),
                                // ));
                                // if refresh_btn.clicked() {}
            
                                // config button
                                let config_btn = ui.add(Button::new(
                                    RichText::new("🛠")
                                        .text_style(egui::TextStyle::Body)
                                        .color(Color32::LIGHT_BLUE),
                                ));
            
                                if config_btn.clicked() {
                                    self.setting.open = !self.setting.open
                                }
                            });
                        });
                        ui.add_space(2.0);
                        ui.add(Separator::default().spacing(0.0));

                            

                    CollapsingHeader::new("DI")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = Vec2::ZERO;
                            ui.horizontal_wrapped(|ui| {
                                for di in &dcc.io_data.di_list {
                                    let color = if di.parse_value == 1 {
                                        Color32::YELLOW
                                    } else {
                                        Color32::GRAY
                                    };
                                    let _ =
                                        ui.button(RichText::new(di.index.to_string()).color(color));
                                    ui.add_space(3.0);
                                }
                            });
                        });
                    CollapsingHeader::new("DO")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = Vec2::ZERO;
                            ui.horizontal_wrapped(|ui| {
                                for o in &dcc.io_data.do_list {
                                    let color = if o.parse_value == 1 {
                                        Color32::YELLOW
                                    } else {
                                        Color32::GRAY
                                    };
                                    let do_ctr =
                                        ui.button(RichText::new(o.index.to_string()
                                    ).color(color))
                                    .on_hover_cursor(CursorIcon::PointingHand);
                                    if do_ctr.clicked() {
                                        if let Some(tx) = &self.front_tx {
                                            let mut clone = o.clone();

                                            clone.parse_value = (o.parse_value + 1) & 0x01;

                                            tx.send(ToBackend::DccCmd(
                                                dcc.addr.clone(),
                                                DccCmd::DO(clone),
                                            ))
                                            .expect("Failed sending DO event.");
                                        }
                                    }
                                    ui.add_space(5.0);
                                }
                            });
                        });
                    CollapsingHeader::new("AI")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                for o in &dcc.io_data.ai_list {
                                    let _ = ui.label(
                                        RichText::new(o.parse_value.to_string())
                                            .color(Color32::WHITE),
                                    );

                                    ui.label(RichText::new("mA").color(Color32::YELLOW));
                                }
                            });
                        });

                    CollapsingHeader::new("AO")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                for o in dcc.io_data.ao_list.iter_mut() {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(Color32::YELLOW, o.index.to_string());
                                        let ao_ctr = ui.add(
                                            Slider::new(&mut o.parse_value, 0.0..=20.0)
                                                .step_by(0.01)
                                                .suffix(" mA"),
                                        );
                                        if ao_ctr.drag_released() {
                                            if let Some(tx) = &self.front_tx {
                                                let mut clone = o.clone();

                                                clone.parse_value = o.parse_value;
                                                tx.send(ToBackend::DccCmd(
                                                    dcc.addr.clone(),
                                                    DccCmd::AO(clone),
                                                ))
                                                .expect("Failed sending AO Ctr event.");
                                            }
                                        }
                                    });
                                }
                            });
                        });

                    CollapsingHeader::new("VI")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                for o in &dcc.io_data.via_list {
                                    let _ = ui.label(
                                        RichText::new(o.parse_value.to_string())
                                            .color(Color32::RED),
                                    );
                                    ui.label(RichText::new("V").color(Color32::BLUE));
                                    ui.add_space(3.0);
                                }
                                ui.add_space(3.0);
                                for o in &dcc.io_data.vir_list {
                                    let _ = ui.label(
                                        RichText::new(o.parse_value.to_string())
                                            .color(Color32::GREEN),
                                    );
                                    ui.label(RichText::new("Ω").color(Color32::KHAKI));
                                    ui.add_space(3.0);
                                }
                            });
                        });

                    CollapsingHeader::new("VO")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                for o in dcc.io_data.vo_list.iter_mut() {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(Color32::YELLOW, o.index.to_string());
                                        let vo_ctr =  ui.add(
                                        Slider::new(&mut o.parse_value, 0.00..=10.00)
                                            .step_by(0.01)
                                         
                                            .suffix(" V"),
                                    );
                                    if vo_ctr.drag_released() {
                                        if let Some(tx) = &self.front_tx {
                                            let mut clone = o.clone();

                                            clone.parse_value = o.parse_value;
                                            tx.send(ToBackend::DccCmd(
                                                dcc.addr.clone(),
                                                DccCmd::VO(clone),
                                            ))
                                            .expect("Failed sending AO Ctr event.");
                                        }
                                    }
                                });
                                }
                            });
                        });
                });
        }
    }

    // fn render_stocks(&self, ctx: &Context, ui: &mut eframe::egui::Ui) {
    //     ui.add_space(2.0);
    //     Grid::new("stock_grid")
    //         .max_col_width(60.0)
    //         // .min_row_height(18.0)
    //         .striped(true)
    //         .show(ui, |ui| {
    //             for stock in &self.stocks {
    //                 ui.centered_and_justified(|ui| {
    //                     ui.add(Label::new(
    //                         RichText::new(stock.name.to_string()).text_style(egui::TextStyle::Body),
    //                     ));
    //                 });
    //                 ui.centered_and_justified(|ui| {
    //                     ui.add(Label::new(
    //                         RichText::new(stock.new.to_string()).text_style(egui::TextStyle::Body),
    //                     ));
    //                 });
    //                 let color = if self.setting.show_color {
    //                     match stock.rise_per {
    //                         p if p < 0.0 => Color32::GREEN,
    //                         n if n > 0.0 => Color32::RED,
    //                         _ => Color32::WHITE,
    //                     }
    //                 } else {
    //                     Color32::WHITE
    //                 };
    //                 ui.centered_and_justified(|ui| {
    //                     ui.add(Label::new(
    //                         RichText::new(stock.rise_per.to_string())
    //                             .text_style(egui::TextStyle::Body)
    //                             .color(color),
    //                     ))
    //                 });
    //                 ui.centered_and_justified(|ui| {
    //                     let bids_bars = stock
    //                         .bids
    //                         .iter()
    //                         .map(|(v, p)| Bar::new((p - stock.new).into(), *v as f64).width(0.0080))
    //                         .collect();

    //                     let bid_chart = BarChart::new(bids_bars).color(Color32::LIGHT_GREEN);

    //                     let asks_bar = stock
    //                         .asks
    //                         .iter()
    //                         .map(|(v, p)| Bar::new((p - stock.new).into(), *v as f64).width(0.0080))
    //                         .collect();
    //                     let ask_chart = BarChart::new(asks_bar).color(Color32::YELLOW);

    //                     let plot = Plot::new(stock.code.to_string())
    //                         .allow_zoom(false)
    //                         .allow_drag(false)
    //                         .allow_scroll(false)
    //                         .show_axes([false, false])
    //                         .height(18.0)
    //                         .width(54.0)
    //                         .center_x_axis(true)
    //                         .data_aspect(0.0001) //    0.01 / 1000
    //                         .show(ui, |plot_ui| {
    //                             plot_ui.bar_chart(bid_chart);
    //                             plot_ui.bar_chart(ask_chart)
    //                         })
    //                         .response;

    //                     plot.on_hover_ui(|ui| {
    //                         ui.horizontal(|ui| {
    //                             ui.group(|ui| {
    //                                 ui.set_max_size(Vec2::new(60.0, 240.0));
    //                                 ui.vertical(|ui| {
    //                                     for (v, p) in stock.asks.iter().rev() {
    //                                         ui.label(format!("{}({})", p, v));
    //                                     }
    //                                     ui.add(Separator::default().spacing(0.0));
    //                                     for (v, p) in &stock.bids {
    //                                         ui.label(format!("{}({})", p, v));
    //                                     }
    //                                 });
    //                             });
    //                             ui.group(|ui| {
    //                                 ui.vertical(|ui| {
    //                                     let bids = stock
    //                                         .bids
    //                                         .iter()
    //                                         .map(|(v, p)| {
    //                                             Bar::new((p - stock.new).into(), *v as f64)
    //                                                 .width(0.0080)
    //                                         })
    //                                         .collect();
    //                                     let asks = stock
    //                                         .asks
    //                                         .iter()
    //                                         .map(|(v, p)| {
    //                                             Bar::new((p - stock.new).into(), *v as f64)
    //                                                 .width(0.0080)
    //                                         })
    //                                         .collect();

    //                                     let bid_chart =
    //                                         BarChart::new(bids).color(Color32::GREEN).horizontal();
    //                                     let ask_chart =
    //                                         BarChart::new(asks).color(Color32::YELLOW).horizontal();

    //                                     Plot::new(stock.code.to_string())
    //                                         .show_axes([false, true])
    //                                         .width(80.0)
    //                                         .center_y_axis(true)
    //                                         .data_aspect(10000.0)
    //                                         .show(ui, |plot_ui| {
    //                                             plot_ui.bar_chart(bid_chart);
    //                                             plot_ui.bar_chart(ask_chart)
    //                                         })
    //                                         .response;
    //                                 })
    //                             })
    //                         });
    //                     });
    //                 });

    //                 ui.end_row();
    //             }
    //         });
    // }

    fn setting_panel(&mut self, ctx: &eframe::egui::Context) {
        if self.setting.open {
            SidePanel::right("setting")
                .default_width(120.0)
                .frame(Frame::none().fill(Color32::from_black_alpha(255)))
                .show(ctx, |ui| {
                    menu::bar(ui, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.label(RichText::new("⚙ setting").color(Color32::LIGHT_BLUE));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let close_btn = ui.add(Button::new(
                                    RichText::new("\u{2bab}")
                                        .text_style(TextStyle::Body)
                                        .color(Color32::GRAY),
                                ));
                                if close_btn.clicked() {
                                    self.setting.open = false
                                }
                            });
                        });
                    });

                    self.setting_panel_contents(ui);
                });
        }
    }

    fn setting_panel_contents(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 50.0;
            ui.label(RichText::new("🕘").color(Color32::GREEN));
            let interval_slider = ui.add(
                Slider::new(&mut self.setting.interval, 1..=30)
                    .suffix(" s")
                    .step_by(1.0),
            );
            if interval_slider.changed() {
                if let Some(tx) = &self.front_tx {
                    tx.send(ToBackend::SetInterval(self.setting.interval))
                        .expect("Failed sending  modify interval event .");
                }
            }
        });
        ui.add(Separator::default().spacing(0.0));

        ui.horizontal(|ui| {
            ui.label(RichText::new("🎨").color(Color32::GOLD));
            ui.checkbox(&mut self.setting.show_color, "color");
        });
        ui.add(Separator::default().spacing(0.0));

        ui.horizontal(|ui| {
            ui.label(RichText::new("📓").color(Color32::LIGHT_BLUE));
            CollapsingHeader::new("stocks")
                .default_open(false)
                .show(ui, |ui| {
                    // for s in &mut self.stocks {
                    //     ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    //         ui.label(RichText::new(s.name.to_string()).color(Color32::LIGHT_BLUE));
                    //         ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    //             ui.add_space(3.0);
                    //             let close_btn = ui.add(Button::new(
                    //                 RichText::new("❌")
                    //                     .text_style(TextStyle::Body)
                    //                     .color(Color32::RED),
                    //             ));
                    //             if close_btn.clicked() {
                    //                 if let Some(tx) = &self.front_tx {
                    //                     tx.send(ToBackend::StockDel(s.code.clone()))
                    //                         .expect("Failed sending  SetCodel event .");
                    //                 };
                    //             }
                    //         });
                    //     });
                    // }
                });
        });
        ui.add(Separator::default().spacing(0.0));
        ui.horizontal(|ui| {
            ui.label(RichText::new("➕").color(Color32::LIGHT_GRAY));
            let Self {
                setting:
                    Setting {
                        open,
                        adding_code: code,
                        ..
                    },
                ..
            } = self;
            let text_color = if true { Color32::GREEN } else { Color32::WHITE };
            let response = ui.add_sized(
                ui.available_size() - Vec2 { x: 2.0, y: 0.0 },
                egui::TextEdit::singleline(code).text_color(text_color),
            );

            if response.lost_focus()
            //  && ui.input().key_pressed(egui::Key::Enter)
            {};
        });
        ui.add(Separator::default().spacing(0.0));
    }
}

impl App for DccTestApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        if let Some(rx) = &self.back_rx {
            match rx.try_recv() {
                Ok(message) => match message {
                    ToFrontend::DccPacket(addr, pkt) => {
                        let dev_id = pkt.header.device_id;
                        let dcc = self.dccs.entry(dev_id).or_insert(Dcc {
                            addr: addr,
                            ..Dcc::default()
                        });
                        match pkt.body {
                            crate::model::DccPacketBody::HeartBeat(io_data) => {
                                dcc.io_data = io_data
                            }
                            _ => {}
                        }
                    }
                },
                Err(err) => {
                    let _ = err;
                }
            }
        }
      //  self.render_top_panel(ctx, frame);

        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                //  self.render_stocks(ctx, ui);
                self.render_dccs(ctx, ui)
            });
       // self.setting_panel(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.setting);
    }
}
