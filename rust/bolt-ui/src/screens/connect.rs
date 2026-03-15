use eframe::egui;

use crate::app::BoltApp;
use crate::state::*;
use crate::theme;

/// EN3d: Host/Join connect screen with real daemon launcher.
pub fn show(ui: &mut egui::Ui, app: &mut BoltApp) {
    ui.add_space(theme::SPACING_XL);

    // Show prerequisite errors
    if let Some(err) = &app.prereq_error {
        ui.colored_label(theme::ERROR, format!("Prerequisite: {err}"));
        ui.add_space(theme::SPACING_MD);
    }

    // Mode selector (Host / Join tabs)
    ui.horizontal(|ui| {
        let host_active = app.mode == ConnectMode::Host;
        let join_active = app.mode == ConnectMode::Join;
        let can_switch = app.connection.can_retry();

        if ui
            .selectable_label(
                host_active,
                egui::RichText::new("Host")
                    .size(theme::FONT_SIZE_HEADING)
                    .color(if host_active { theme::ACCENT } else { theme::TEXT_SECONDARY }),
            )
            .clicked()
            && can_switch
        {
            app.mode = ConnectMode::Host;
        }

        ui.add_space(theme::SPACING_LG);

        if ui
            .selectable_label(
                join_active,
                egui::RichText::new("Join")
                    .size(theme::FONT_SIZE_HEADING)
                    .color(if join_active { theme::ACCENT } else { theme::TEXT_SECONDARY }),
            )
            .clicked()
            && can_switch
        {
            app.mode = ConnectMode::Join;
        }
    });

    ui.add_space(theme::SPACING_LG);
    ui.separator();
    ui.add_space(theme::SPACING_LG);

    match app.mode {
        ConnectMode::Host => show_host(ui, app),
        ConnectMode::Join => show_join(ui, app),
    }

    // Status bar
    ui.add_space(theme::SPACING_XL);
    let status_color = match &app.connection {
        ConnectionState::Idle => theme::TEXT_MUTED,
        ConnectionState::Connecting { .. } => theme::WARNING,
        ConnectionState::Connected => theme::SUCCESS,
        ConnectionState::TimedOut => theme::ERROR,
        ConnectionState::Error(_) => theme::ERROR,
    };
    let status_text = match &app.connection {
        ConnectionState::Error(msg) => format!("Error: {msg}"),
        other => other.status_text().to_string(),
    };
    ui.colored_label(status_color, &status_text);

    // Show daemon stderr for diagnostics
    if let Some(proc) = &app.daemon_proc {
        let lines = proc.recent_stderr(3);
        if !lines.is_empty() {
            ui.add_space(theme::SPACING_SM);
            for line in &lines {
                ui.colored_label(
                    theme::TEXT_MUTED,
                    egui::RichText::new(line).size(theme::FONT_SIZE_SMALL),
                );
            }
        }
    }
}

fn show_host(ui: &mut egui::Ui, app: &mut BoltApp) {
    ui.vertical_centered(|ui| {
        if app.host_info.is_none() && app.connection == ConnectionState::Idle {
            // Step 1: Generate host info
            let btn = egui::Button::new(
                egui::RichText::new("Create Session")
                    .size(theme::FONT_SIZE_BODY)
                    .color(theme::ACCENT_FG),
            )
            .fill(theme::ACCENT)
            .corner_radius(theme::ROUNDING);

            if ui.add_enabled(app.prereq_error.is_none(), btn).clicked() {
                app.start_host();
            }
        }

        if let Some(info) = &app.host_info {
            // Step 2: Show connection details for sharing
            ui.label(
                egui::RichText::new("Share these with your peer:")
                    .size(theme::FONT_SIZE_BODY)
                    .color(theme::TEXT_SECONDARY),
            );
            ui.add_space(theme::SPACING_MD);

            egui::Frame::NONE
                .fill(theme::PANEL_BG)
                .corner_radius(theme::ROUNDING)
                .inner_margin(theme::SPACING_LG)
                .stroke(egui::Stroke::new(1.0, theme::ACCENT))
                .show(ui, |ui| {
                    ui.set_min_width(280.0);
                    show_field(ui, "Room", &info.room);
                    show_field(ui, "Session", &info.session);
                    show_field(ui, "Code", &info.peer_code);
                });

            // Step 3: Wait for joiner — host needs joiner's code to spawn daemon
            if matches!(app.connection, ConnectionState::Idle) {
                ui.add_space(theme::SPACING_LG);
                ui.label(
                    egui::RichText::new("Enter joiner's peer code to start:")
                        .size(theme::FONT_SIZE_BODY)
                        .color(theme::TEXT_SECONDARY),
                );
                ui.add_space(theme::SPACING_SM);

                // We need a mutable reference through app, use a temp buffer
                let mut joiner_input = app.join_peer_code.clone();
                let input = egui::TextEdit::singleline(&mut joiner_input)
                    .hint_text("Joiner's code")
                    .font(egui::TextStyle::Monospace)
                    .desired_width(150.0)
                    .char_limit(bolt_core::constants::PEER_CODE_LENGTH);
                ui.add(input);
                app.join_peer_code = joiner_input;

                ui.add_space(theme::SPACING_MD);

                let start_btn = egui::Button::new(
                    egui::RichText::new("Start Listening")
                        .size(theme::FONT_SIZE_BODY)
                        .color(theme::ACCENT_FG),
                )
                .fill(theme::ACCENT)
                .corner_radius(theme::ROUNDING);

                let ready = app.join_peer_code.len() == bolt_core::constants::PEER_CODE_LENGTH;
                if ui.add_enabled(ready, start_btn).clicked() {
                    let code = app.join_peer_code.clone();
                    app.start_host_with_joiner(&code);
                }
            }
        }

        // Cancel button while connecting
        if app.connection.can_cancel() {
            ui.add_space(theme::SPACING_MD);
            let cancel = egui::Button::new(
                egui::RichText::new("Cancel").size(theme::FONT_SIZE_BODY).color(theme::TEXT_PRIMARY),
            )
            .fill(theme::ERROR)
            .corner_radius(theme::ROUNDING);
            if ui.add(cancel).clicked() {
                app.cancel_connect();
                app.host_info = None;
            }
        }
    });
}

fn show_join(ui: &mut egui::Ui, app: &mut BoltApp) {
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("Your Peer Code")
                .size(theme::FONT_SIZE_SMALL)
                .color(theme::TEXT_SECONDARY),
        );
        ui.label(
            egui::RichText::new(&app.local_peer_code)
                .size(theme::FONT_SIZE_TITLE)
                .color(theme::ACCENT)
                .monospace(),
        );

        ui.add_space(theme::SPACING_XL);

        ui.label(
            egui::RichText::new("Enter host's connection details:")
                .size(theme::FONT_SIZE_BODY)
                .color(theme::TEXT_SECONDARY),
        );
        ui.add_space(theme::SPACING_MD);

        let is_connecting = matches!(app.connection, ConnectionState::Connecting { .. });

        // Room input
        let mut room = app.join_room.clone();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Room:").color(theme::TEXT_SECONDARY));
            let r = egui::TextEdit::singleline(&mut room)
                .hint_text("e.g. r1a2b3")
                .font(egui::TextStyle::Monospace)
                .desired_width(150.0)
                .interactive(!is_connecting);
            ui.add(r);
        });
        app.join_room = room;

        ui.add_space(theme::SPACING_SM);

        // Session input
        let mut sess = app.join_session.clone();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Session:").color(theme::TEXT_SECONDARY));
            let s = egui::TextEdit::singleline(&mut sess)
                .hint_text("e.g. s4d5e6f7")
                .font(egui::TextStyle::Monospace)
                .desired_width(150.0)
                .interactive(!is_connecting);
            ui.add(s);
        });
        app.join_session = sess;

        ui.add_space(theme::SPACING_SM);

        // Peer code input
        let mut code = app.join_peer_code.clone();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Code:").color(theme::TEXT_SECONDARY));
            let c = egui::TextEdit::singleline(&mut code)
                .hint_text("Host's code")
                .font(egui::TextStyle::Monospace)
                .desired_width(150.0)
                .interactive(!is_connecting);
            ui.add(c);
        });
        app.join_peer_code = code;

        ui.add_space(theme::SPACING_LG);

        if is_connecting {
            let cancel = egui::Button::new(
                egui::RichText::new("Cancel").size(theme::FONT_SIZE_BODY).color(theme::TEXT_PRIMARY),
            )
            .fill(theme::ERROR)
            .corner_radius(theme::ROUNDING);
            if ui.add(cancel).clicked() {
                app.cancel_connect();
            }
        } else {
            let label = if app.connection.can_retry() && !matches!(app.connection, ConnectionState::Idle) {
                "Retry"
            } else {
                "Join"
            };
            let join_btn = egui::Button::new(
                egui::RichText::new(label).size(theme::FONT_SIZE_BODY).color(theme::ACCENT_FG),
            )
            .fill(theme::ACCENT)
            .corner_radius(theme::ROUNDING);

            let ready = !app.join_room.is_empty()
                && !app.join_session.is_empty()
                && !app.join_peer_code.is_empty()
                && app.prereq_error.is_none();

            if ui.add_enabled(ready, join_btn).clicked() {
                app.start_join();
            }
        }
    });
}

fn show_field(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{label}:"))
                .size(theme::FONT_SIZE_BODY)
                .color(theme::TEXT_SECONDARY),
        );
        ui.label(
            egui::RichText::new(value)
                .size(theme::FONT_SIZE_BODY)
                .color(theme::ACCENT)
                .monospace(),
        );
    });
}
