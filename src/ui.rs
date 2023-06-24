use bevy::prelude::*;

use bevy_egui::{
    egui::{self, ScrollArea},
    EguiContexts,
};
use egui::Color32;

use crate::components::{ElevationBundle, Evaporation, Humidity, HumidityReceived, HumiditySent, Overflow, Precipitation, HexCoordinates, Temperature};
use crate::terrain::TileType;

#[derive(Debug, Clone, Default, Resource)]
pub struct SelectedTile {
    pub entity: Option<Entity>,
    pub hex_coordinates: Option<HexCoordinates>,
    pub elevation: Option<ElevationBundle>,
    pub humidity: Option<Humidity>,
    pub temperature: Option<Temperature>,
    pub evaporation: Option<Evaporation>,
    pub precipitation: Option<Precipitation>,
    pub overflow: Option<Overflow>,
    pub tile_type: Option<TileType>,
    pub humidity_received: Option<HumidityReceived>,
    pub humidity_sent: Option<HumiditySent>,
}

pub fn terrain_details(mut egui_contexts: EguiContexts, selected_tile: Res<SelectedTile>) {
    egui::Window::new("Terrain Details").show(egui_contexts.ctx_mut(), |ui| {
        ScrollArea::both().auto_shrink([true; 2]).show(ui, |ui| {
            if let Some(entity) = selected_tile.entity {
                ui.vertical(|ui| {
                    ui.heading("Selected Tile:");
                    ui.separator();
                    if let Some(tile_type) = &selected_tile.tile_type {
                        ui.horizontal(|ui| {
                            ui.label("Tile Type:");
                            ui.label(format!("{:?}", tile_type));
                        });
                    }
                    ui.horizontal(|ui| {
                        ui.label("Entity:");
                        ui.label(format!("{:?}", entity));
                    });
                    // Add more components as needed
                    if let Some(hex_coordinates) = &selected_tile.hex_coordinates {
                        ui.horizontal(|ui| {
                            ui.label("Hex Coordinates:");
                            ui.label(format!("{}", hex_coordinates));
                        });
                    }
                    if let Some(elevation) = &selected_tile.elevation {
                        ui.horizontal(|ui| {
                            ui.label("water elevation");
                            ui.colored_label(Color32::BLUE, format!("{}", elevation.water));
                        });
                        ui.horizontal(|ui| {
                            ui.label("soil elevation");
                            ui.colored_label(Color32::BROWN, format!("{}", elevation.soil));
                        });
                        ui.horizontal(|ui| {
                            ui.label("bedrock elevation");
                            ui.colored_label(Color32::GRAY, format!("{}", elevation.bedrock));
                        });
                    }
                    if let Some(humidity) = &selected_tile.humidity {
                        ui.horizontal(|ui| {
                            ui.label("Humidity:");
                            ui.label(format!("{}", humidity));
                        });
                    }

                    if let Some(temperature) = &selected_tile.temperature {
                        ui.horizontal(|ui| {
                            ui.label("Temperature:");
                            ui.label(format!("{}", temperature));
                        });
                    }
                    if let Some(evaporation) = &selected_tile.evaporation {
                        ui.horizontal(|ui| {
                            ui.label("Evaporation:");
                            ui.label(format!("{}", evaporation));
                        });
                    }
                    if let Some(precipitation) = &selected_tile.precipitation {
                        ui.horizontal(|ui| {
                            ui.label("Precipitation:");
                            ui.label(format!("{}", precipitation));
                        });
                    }
                    if let Some(overflow) = &selected_tile.overflow {
                        ui.horizontal(|ui| {
                            ui.label("Overflow:");
                            ui.label(format!("{}", overflow));
                        });
                    }
                    if let Some(humidity_received) = &selected_tile.humidity_received {
                        ui.horizontal(|ui| {
                            ui.label("Humidity Received:");
                            ui.label(format!("{}", humidity_received.value));
                        });
                    }
                    if let Some(humidity_sent) = &selected_tile.humidity_sent {
                        ui.horizontal(|ui| {
                            ui.label("Humidity Sent:");
                            ui.label(format!("{}", humidity_sent.value));
                        });
                    }
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.label("No tile selected");
                });
            }
        });
    });
}
