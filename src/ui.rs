use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use bevy_egui::{
    egui::{self, ScrollArea},
    EguiContexts,
};
use egui::Color32;

use crate::components::{
    DebugWeatherBundle, ElevationBundle, Evaporation, HexCoordinates, Humidity, HumidityReceived,
    HumiditySent, Overflow, OverflowReceived, Precipitation, Temperature,
};
use crate::terrain::TileType;

pub fn terrain_callback(
    event: Listener<Pointer<Click>>,
    query: Query<(
        Entity,
        &HexCoordinates,
        &ElevationBundle,
        &Humidity,
        &Temperature,
        &TileType,
        &DebugWeatherBundle,
        &Children,
    )>,
    mut selected_tile: ResMut<SelectedTile>,
) {
    for (entity, hex_coordinates, elevation, humidity, temperature, tile_type, weather, parent) in
        query.iter()
    {
        if entity == event.listener() {
            println!("Selected tile: {:?}", entity);
            selected_tile.entity = Some(entity);
            selected_tile.hex_coordinates = Some(hex_coordinates.clone());
            selected_tile.elevation = Some(*elevation);
            selected_tile.humidity = Some(*humidity);
            selected_tile.temperature = Some(*temperature);
            selected_tile.evaporation = Some(weather.evaporation);
            selected_tile.precipitation = Some(weather.precipitation);
            selected_tile.humidity_received = Some(weather.humidity_received);
            selected_tile.humidity_sent = Some(weather.humidity_sent);
            selected_tile.overflow = Some(weather.overflow);
            selected_tile.overflow_received = Some(weather.overflow_received);
            selected_tile.tile_type = Some(*tile_type);
            break;
        }
    }
}

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
    pub overflow_received: Option<OverflowReceived>,
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
                            ui.label("Overflow Sent:");
                            ui.label(format!("{}", overflow));
                        });
                    }
                    if let Some(overflow_received) = &selected_tile.overflow_received {
                        ui.horizontal(|ui| {
                            ui.label("Overflow Received:");
                            ui.label(format!("{}", overflow_received));
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
