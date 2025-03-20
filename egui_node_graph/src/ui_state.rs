use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Arc;

use egui::emath::GuiRounding;
use egui::{Style, Ui, Vec2};
#[cfg(feature = "persistence")]
use serde::{Deserialize, Serialize};

use super::*;

use crate::scale::Scale;

const MIN_ZOOM: f32 = 0.2;
const MAX_ZOOM: f32 = 2.0;

#[derive(Clone)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct GraphEditorState<
    NodeData: NodeDataTrait,
    DataType: DataTypeTrait<UserState>,
    ValueType: WidgetValueTrait,
    NodeTemplate: NodeTemplateTrait,
    UserState: UserStateTrait,
> {
    pub graph: Graph<NodeData, DataType, ValueType, UserState>,
    /// Nodes are drawn in this order. Draw order is important because nodes
    /// that are drawn last are on top.
    pub node_order: Vec<NodeId>,
    /// An ongoing connection interaction: The mouse has dragged away from a
    /// port and the user is holding the click
    pub connection_in_progress: Option<(NodeId, AnyParameterId)>,
    /// The currently selected node. Some interface actions depend on the
    /// currently selected node.
    pub selected_nodes: HashSet<NodeId>,
    /// The mouse drag start position for an ongoing box selection.
    pub ongoing_box_selection: Option<egui::Pos2>,
    /// The position of each node.
    pub node_positions: SecondaryMap<NodeId, egui::Pos2>,
    /// The node finder is used to create new nodes.
    pub node_finder: Option<NodeFinder<NodeTemplate>>,
    /// The panning of the graph viewport.
    pub pan_zoom: PanZoom,
    pub _user_state: PhantomData<fn() -> UserState>,
}

impl<
        NodeData: NodeDataTrait,
        DataType: DataTypeTrait<UserState>,
        ValueType: WidgetValueTrait,
        NodeKind: NodeTemplateTrait,
        UserState: UserStateTrait,
    > GraphEditorState<NodeData, DataType, ValueType, NodeKind, UserState>
{
    pub fn new(default_zoom: f32) -> Self {
        Self {
            pan_zoom: PanZoom::new(default_zoom),
            ..Default::default()
        }
    }

    /// Create a new GraphEditorState from the selected nodes
    pub fn from_selected(&self) -> Self {
        let mut editor_state: Self = self.clone();

        let graph = self.graph.from_nodes(&self.selected_nodes);
        editor_state.graph = graph;
        editor_state.node_finder = None;
        editor_state.connection_in_progress = None;
        editor_state.ongoing_box_selection = None;
        editor_state.selected_nodes.clear();
        editor_state
            .node_order
            .retain(|id| editor_state.graph.nodes.contains_key(*id));
        editor_state
            .node_positions
            .retain(|id, _pos| editor_state.graph.nodes.contains_key(id));

        editor_state
    }
}

impl<
        NodeData: NodeDataTrait,
        DataType: DataTypeTrait<UserState>,
        ValueType: WidgetValueTrait,
        NodeKind: NodeTemplateTrait,
        UserState: UserStateTrait,
    > Default for GraphEditorState<NodeData, DataType, ValueType, NodeKind, UserState>
{
    fn default() -> Self {
        Self {
            graph: Default::default(),
            node_order: Default::default(),
            connection_in_progress: Default::default(),
            selected_nodes: Default::default(),
            ongoing_box_selection: Default::default(),
            node_positions: Default::default(),
            node_finder: Default::default(),
            pan_zoom: Default::default(),
            _user_state: Default::default(),
        }
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct PanZoom {
    pub pan: Vec2,
    pub zoom: f32,
    #[cfg_attr(feature = "persistence", serde(skip, default))]
    pub zoomed_style: Arc<Style>,
    #[cfg_attr(feature = "persistence", serde(skip, default))]
    pub started: bool,
    pub enable_zoom_from_out_of_rect: bool,
}

impl Default for PanZoom {
    fn default() -> Self {
        PanZoom {
            pan: Vec2::ZERO,
            zoom: 1.0,
            zoomed_style: Default::default(),
            started: false,
            enable_zoom_from_out_of_rect: false,
        }
    }
}

impl PanZoom {
    pub fn new(zoom: f32) -> PanZoom {
        let style: Style = Default::default();
        PanZoom {
            pan: Vec2::ZERO,
            zoom,
            zoomed_style: Arc::new(style.scaled(1.0)),
            started: false,
            enable_zoom_from_out_of_rect: false,
        }
    }

    pub fn zoom(&mut self, style: &Arc<Style>, zoom_delta: f32) {
        let new_zoom = (self.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
        self.zoomed_style = Arc::new(style.scaled(new_zoom));
        self.zoom = new_zoom;
    }
}

pub fn show_zoomed<R, F>(
    default_style: Arc<Style>,
    zoomed_style: Arc<Style>,
    ui: &mut Ui,
    add_content: F,
) -> R
where
    F: FnOnce(&mut Ui) -> R,
{
    let mut zoomed_style: Style = (*zoomed_style).clone();

    let pixels_per_point: f32 = ui.painter().pixels_per_point();

    if let Some(override_font_id) = &mut zoomed_style.override_font_id {
        override_font_id.size = override_font_id
            .size
            .round_to_pixel_center(pixels_per_point);
    }

    for text_style in zoomed_style.text_styles.values_mut() {
        text_style.size = text_style.size.round_to_pixel_center(pixels_per_point);
    }

    *ui.style_mut() = zoomed_style;
    let response = add_content(ui);
    *ui.style_mut() = (*default_style).clone();

    response
}
