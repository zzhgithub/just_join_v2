use bevy::prelude::{Plugin, ResMut, Resource, Update};
use bevy_egui::EguiContexts;
use egui_notify::Toasts;

#[derive(Resource, Default)]
pub struct Notification {
    pub toasts: Toasts,
}

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Notification::default());
        app.add_systems(Update, update_notifications);
    }
}

fn update_notifications(mut contexts: EguiContexts, mut notification: ResMut<Notification>) {
    let ctx = contexts.ctx_mut();
    notification.toasts.show(ctx);
}
