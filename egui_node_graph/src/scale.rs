use egui::epaint::{CornerRadius, CornerRadiusF32, Margin, Marginf, Shadow};
use egui::{
    style::{Interaction, ScrollStyle, Visuals, WidgetVisuals, Widgets},
    Spacing, Stroke, Style, Vec2,
};

// Copied from https://github.com/gzp-crey/shine - then modified

pub trait Scale {
    fn scale(&mut self, amount: f32);

    fn scaled(&self, amount: f32) -> Self
    where
        Self: Clone,
    {
        let mut scaled = self.clone();
        scaled.scale(amount);
        scaled
    }
}

impl Scale for Vec2 {
    fn scale(&mut self, amount: f32) {
        self.x *= amount;
        self.y *= amount;
    }
}

impl Scale for Margin {
    fn scale(&mut self, amount: f32) {
        self.left = (self.left as f32 * amount).round().clamp(-128., 127.) as i8;
        self.right = (self.right as f32 * amount).round().clamp(-128., 127.) as i8;
        self.top = (self.top as f32 * amount).round().clamp(-128., 127.) as i8;
        self.bottom = (self.bottom as f32 * amount).round().clamp(-128., 127.) as i8;
    }
}

impl Scale for Marginf {
    fn scale(&mut self, amount: f32) {
        self.left *= amount;
        self.right *= amount;
        self.top *= amount;
        self.bottom *= amount;
    }
}

impl Scale for CornerRadius {
    fn scale(&mut self, amount: f32) {
        self.ne = (self.ne as f32 * amount).round().clamp(0., 255.) as u8;
        self.nw = (self.nw as f32 * amount).round().clamp(0., 255.) as u8;
        self.se = (self.se as f32 * amount).round().clamp(0., 255.) as u8;
        self.sw = (self.sw as f32 * amount).round().clamp(0., 255.) as u8;
    }
}

impl Scale for CornerRadiusF32 {
    fn scale(&mut self, amount: f32) {
        self.ne *= amount;
        self.nw *= amount;
        self.se *= amount;
        self.sw *= amount;
    }
}

impl Scale for Stroke {
    fn scale(&mut self, amount: f32) {
        self.width *= amount;
    }
}

impl Scale for Shadow {
    fn scale(&mut self, amount: f32) {
        self.spread = (self.spread as f32 * amount).round().clamp(0., 255.) as u8;
    }
}

impl Scale for WidgetVisuals {
    fn scale(&mut self, amount: f32) {
        self.bg_stroke.scale(amount);
        self.fg_stroke.scale(amount);
        self.corner_radius.scale(amount);
        self.expansion *= amount;
    }
}

impl Scale for ScrollStyle {
    fn scale(&mut self, amount: f32) {
        self.bar_width *= amount;
        self.handle_min_length *= amount;
        self.bar_inner_margin *= amount;
        self.bar_outer_margin *= amount;
        self.floating_width *= amount;
        self.floating_allocated_width *= amount;
    }
}

impl Scale for Spacing {
    fn scale(&mut self, amount: f32) {
        self.item_spacing.scale(amount);
        self.window_margin.scale(amount);
        self.button_padding.scale(amount);
        self.menu_margin.scale(amount);
        self.indent *= amount;
        self.interact_size.scale(amount);
        self.slider_width *= amount;
        self.slider_rail_height *= amount;
        self.combo_width *= amount;
        self.text_edit_width *= amount;
        self.icon_width *= amount;
        self.icon_width_inner *= amount;
        self.icon_spacing *= amount;
        self.default_area_size *= amount;
        self.tooltip_width *= amount;
        self.menu_width *= amount;
        self.menu_spacing *= amount;
        self.combo_height *= amount;
        self.scroll.scale(amount);
    }
}

impl Scale for Interaction {
    fn scale(&mut self, amount: f32) {
        self.resize_grab_radius_side *= amount;
        self.resize_grab_radius_corner *= amount;
    }
}

impl Scale for Widgets {
    fn scale(&mut self, amount: f32) {
        self.noninteractive.scale(amount);
        self.inactive.scale(amount);
        self.hovered.scale(amount);
        self.active.scale(amount);
        self.open.scale(amount);
    }
}

impl Scale for Visuals {
    fn scale(&mut self, amount: f32) {
        self.widgets.scale(amount);
        self.selection.stroke.scale(amount);
        self.resize_corner_size *= amount;
        self.menu_corner_radius.scale(amount);
        self.text_cursor.stroke.scale(amount);
        self.clip_rect_margin *= amount;
        self.window_corner_radius.scale(amount);
        self.window_shadow.scale(amount);
        self.popup_shadow.scale(amount);
    }
}

impl Scale for Style {
    fn scale(&mut self, amount: f32) {
        if let Some(ov_font_id) = &mut self.override_font_id {
            ov_font_id.size *= amount;
        }

        for text_style in self.text_styles.values_mut() {
            text_style.size *= amount;
        }

        self.spacing.scale(amount);
        self.interaction.scale(amount);
        self.visuals.scale(amount);
    }
}
