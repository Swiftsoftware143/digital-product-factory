//! License activation dialog

use egui::*;
use crate::app::DpfApp;

pub fn show(app: &mut DpfApp, ctx: &Context) {
    Window::new("License")
        .collapsible(false)
        .resizable(false)
        .default_size([400.0, 300.0])
        .show(ctx, |ui| {
            ui.heading("License Activation");
            ui.separator();
            
            if app.license_manager.is_licensed() {
                // Show current license info
                if let Some(license) = app.license_manager.current_license() {
                    ui.label(format!("License Key: {}", license.key));
                    ui.label(format!("Tier: {:?}", license.tier));
                    ui.label(format!("Devices: {}/{}", 
                        license.activated_devices.len(), 
                        license.max_devices));
                    
                    if ui.button("Deactivate").clicked() {
                        app.license_manager.deactivate();
                    }
                }
            } else {
                // Activation form
                ui.label("Enter your license key:");
                
                let mut key_input = String::new();
                ui.add(egui::TextEdit::singleline(&mut key_input).hint_text("XXXX-XXXX-XXXX-XXXX"));
                
                if ui.button("Activate").clicked() {
                    // TODO: Validate and activate
                }
                
                ui.separator();
                
                ui.label("Don't have a license?");
                ui.hyperlink_to("Purchase", "https://your-store.com");
            }
            
            ui.separator();
            
            if ui.button("Close").clicked() {
                app.show_license_dialog = false;
            }
        });
}
