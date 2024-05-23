use std::collections::{HashMap, HashSet};

use super::*;

impl<
        NodeData: NodeDataTrait,
        DataType: DataTypeTrait<UserState>,
        ValueType: WidgetValueTrait,
        UserState: UserStateTrait,
    > Graph<NodeData, DataType, ValueType, UserState>
{
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::default(),
            inputs: SlotMap::default(),
            outputs: SlotMap::default(),
            connections: SecondaryMap::default(),
        }
    }

    pub fn add_node(&mut self, label: String, user_data: NodeData) -> NodeId {
        self.nodes.insert_with_key(|node_id| {
            Node {
                id: node_id,
                label,
                // These get filled in later by the user function
                inputs: Vec::default(),
                outputs: Vec::default(),
                user_data,
            }
        })
    }

    pub fn merge(&mut self, other: &Self) -> HashMap<NodeId, NodeId> {
        let mut old_to_new_node_ids = HashMap::<NodeId, NodeId>::new();
        let mut old_connections = HashMap::<OutputId, HashSet<InputId>>::new();
        let mut old_to_new_inputs = HashMap::<InputId, InputId>::new();
        let mut old_to_new_outputs = HashMap::<OutputId, OutputId>::new();
        for node_id in other.iter_nodes() {
            if let Some(node_to_merge) = other.nodes.get(node_id) {
                let mut duplicated_inputs: Vec<(String, InputId)> = node_to_merge.inputs.clone();
                let mut duplicated_outputs: Vec<(String, OutputId)> = node_to_merge.outputs.clone();
                let mut duplicate_node: Node<NodeData> = node_to_merge.clone();

                // Create and set the new node's id
                let new_node_id: NodeId = self.nodes.insert_with_key(|node_id| {
                    duplicate_node.id = node_id;
                    duplicate_node
                });

                // Update the cloned inputs with new ids, and the new node's id
                for (_label, input_id) in duplicated_inputs.iter_mut() {
                    if let Some(input) = other.inputs.get(*input_id) {
                        let mut duplicated_input = (*input).clone();
                        duplicated_input.node = new_node_id;
                        let duplicate_id = self.inputs.insert_with_key(|duplicate_id| {
                            duplicated_input.id = duplicate_id;
                            duplicated_input
                        });
                        if let Some(output_id) = other.connections.get(*input_id) {
                            // Maintain a list of connections to duplicate
                            if let Some(connected_inputs) = old_connections.get_mut(output_id) {
                                connected_inputs.insert(*input_id);
                            } else {
                                let mut inputs = HashSet::<InputId>::new();
                                inputs.insert(*input_id);
                                old_connections.insert(*output_id, inputs);
                            }
                            // Don't need to store an input if it isn't connected
                            // but if it is, we need a LUT for the original to new ids
                            old_to_new_inputs.insert(*input_id, duplicate_id);
                        }
                        *input_id = duplicate_id;
                    }
                }

                // Update the cloned outputs with new ids, and the new node's id
                for (_label, output_id) in duplicated_outputs.iter_mut() {
                    if let Some(output) = other.outputs.get(*output_id) {
                        let mut duplicated_output = (*output).clone();
                        duplicated_output.node = new_node_id;
                        let duplicate_id = self.outputs.insert_with_key(|duplicate_id| {
                            duplicated_output.id = duplicate_id;
                            duplicated_output
                        });
                        // Keep a LUT of old to new output ids for connecting new nodes
                        old_to_new_outputs.insert(*output_id, duplicate_id);
                        *output_id = duplicate_id;
                    }
                }

                // Update the new node with its new inputs and outputs
                if let Some(node) = self.nodes.get_mut(new_node_id) {
                    node.inputs = duplicated_inputs;
                    node.outputs = duplicated_outputs;

                    old_to_new_node_ids.insert(node_id, new_node_id);
                }
            }
        }

        // Form equivalent connections
        for (old_output_id, old_connected_input_ids) in old_connections.iter() {
            if let Some(new_output_id) = old_to_new_outputs.get(old_output_id) {
                for old_input_id in old_connected_input_ids.iter() {
                    if let Some(new_input_id) = old_to_new_inputs.get(old_input_id) {
                        self.connections.insert(*new_input_id, *new_output_id);
                    }
                }
            }
        }

        old_to_new_node_ids
    }

    pub fn from_nodes(&self, node_ids: &HashSet<NodeId>) -> Self {
        let mut new_graph: Self = self.clone();

        for node_id in self.iter_nodes() {
            if node_ids.contains(&node_id) {
                continue;
            }
            let (_node, disconnections) = new_graph.remove_node(node_id);
            for (input_id, _output_id) in disconnections.iter() {
                new_graph.remove_connection(*input_id);
            }
        }

        new_graph
    }

    /// Duplicate nodes and return the new node's ids in the equivalent order
    pub fn duplicate_nodes(&mut self, node_ids: &HashSet<NodeId>) -> HashMap<NodeId, NodeId> {
        let mut old_to_new_node_ids = HashMap::<NodeId, NodeId>::new();
        let mut old_connections = HashMap::<OutputId, HashSet<InputId>>::new();
        let mut old_to_new_inputs = HashMap::<InputId, InputId>::new();
        let mut old_to_new_outputs = HashMap::<OutputId, OutputId>::new();
        for node_id in node_ids.iter() {
            if let Some(node_to_duplicate) = self.nodes.get(*node_id) {
                let mut duplicated_inputs: Vec<(String, InputId)> =
                    node_to_duplicate.inputs.clone();
                let mut duplicated_outputs: Vec<(String, OutputId)> =
                    node_to_duplicate.outputs.clone();
                let mut duplicate_node: Node<NodeData> = node_to_duplicate.clone();

                // Create and set the new node's id
                let new_node_id: NodeId = self.nodes.insert_with_key(|node_id| {
                    duplicate_node.id = node_id;
                    duplicate_node
                });

                // Update the cloned inputs with new ids, and the new node's id
                for (_label, input_id) in duplicated_inputs.iter_mut() {
                    if let Some(input) = self.inputs.get(*input_id) {
                        let mut duplicated_input = (*input).clone();
                        duplicated_input.node = new_node_id;
                        let duplicate_id = self.inputs.insert_with_key(|duplicate_id| {
                            duplicated_input.id = duplicate_id;
                            duplicated_input
                        });
                        if let Some(output_id) = self.connections.get(*input_id) {
                            // Maintain a list of connections to duplicate
                            if let Some(connected_inputs) = old_connections.get_mut(output_id) {
                                connected_inputs.insert(*input_id);
                            } else {
                                let mut inputs = HashSet::<InputId>::new();
                                inputs.insert(*input_id);
                                old_connections.insert(*output_id, inputs);
                            }
                            // Don't need to store an input if it isn't connected
                            // but if it is, we need a LUT for the original to new ids
                            old_to_new_inputs.insert(*input_id, duplicate_id);
                        }
                        *input_id = duplicate_id;
                    }
                }

                // Update the cloned outputs with new ids, and the new node's id
                for (_label, output_id) in duplicated_outputs.iter_mut() {
                    if let Some(output) = self.outputs.get(*output_id) {
                        let mut duplicated_output = (*output).clone();
                        duplicated_output.node = new_node_id;
                        let duplicate_id = self.outputs.insert_with_key(|duplicate_id| {
                            duplicated_output.id = duplicate_id;
                            duplicated_output
                        });
                        // Keep a LUT of old to new output ids for connecting new nodes
                        old_to_new_outputs.insert(*output_id, duplicate_id);
                        *output_id = duplicate_id;
                    }
                }

                // Update the new node with its new inputs and outputs
                if let Some(node) = self.nodes.get_mut(new_node_id) {
                    node.inputs = duplicated_inputs;
                    node.outputs = duplicated_outputs;

                    old_to_new_node_ids.insert(*node_id, new_node_id);
                }
            }
        }

        // Form equivalent connections
        for (old_output_id, old_connected_input_ids) in old_connections.iter() {
            if let Some(new_output_id) = old_to_new_outputs.get(old_output_id) {
                for old_input_id in old_connected_input_ids.iter() {
                    if let Some(new_input_id) = old_to_new_inputs.get(old_input_id) {
                        self.connections.insert(*new_input_id, *new_output_id);
                    }
                }
            }
        }

        old_to_new_node_ids
    }

    pub fn add_input_param(
        &mut self,
        node_id: NodeId,
        name: String,
        typ: DataType,
        value: ValueType,
        kind: InputParamKind,
        shown_inline: bool,
    ) -> InputId {
        let input_id = self.inputs.insert_with_key(|input_id| {
            InputParam::new(input_id, typ, value, kind, node_id, shown_inline)
        });
        self.nodes[node_id].inputs.push((name, input_id));
        input_id
    }

    pub fn remove_input_param(&mut self, param: InputId) {
        let node = self[param].node;
        self[node].inputs.retain(|(_, id)| *id != param);
        self.inputs.remove(param);
        self.connections.retain(|i, _| i != param);
    }

    pub fn remove_output_param(&mut self, param: OutputId) {
        let node = self[param].node;
        self[node].outputs.retain(|(_, id)| *id != param);
        self.outputs.remove(param);
        self.connections.retain(|_, o| *o != param);
    }

    pub fn add_output_param(&mut self, node_id: NodeId, name: String, typ: DataType) -> OutputId {
        let output_id = self
            .outputs
            .insert_with_key(|output_id| OutputParam::new(output_id, typ, node_id));
        self.nodes[node_id].outputs.push((name, output_id));
        output_id
    }

    /// Removes a node from the graph with given `node_id`. This also removes
    /// any incoming or outgoing connections from that node
    ///
    /// This function returns the list of connections that has been removed
    /// after deleting this node as input-output pairs. Note that one of the two
    /// ids in the pair (the one on `node_id`'s end) will be invalid after
    /// calling this function.
    pub fn remove_node(&mut self, node_id: NodeId) -> (Node<NodeData>, Vec<(InputId, OutputId)>) {
        let mut disconnect_events = vec![];

        self.connections.retain(|i, o| {
            if self.outputs[*o].node == node_id || self.inputs[i].node == node_id {
                disconnect_events.push((i, *o));
                false
            } else {
                true
            }
        });

        // NOTE: Collect is needed because we can't borrow the input ids while
        // we remove them inside the loop.
        for input in self[node_id].input_ids().collect::<SVec<_>>() {
            self.inputs.remove(input);
        }
        for output in self[node_id].output_ids().collect::<SVec<_>>() {
            self.outputs.remove(output);
        }
        let removed_node = self.nodes.remove(node_id).expect("Node should exist");

        (removed_node, disconnect_events)
    }

    pub fn remove_connection(&mut self, input_id: InputId) -> Option<OutputId> {
        self.connections.remove(input_id)
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.iter().map(|(id, _)| id)
    }

    pub fn add_connection(&mut self, output: OutputId, input: InputId) {
        self.connections.insert(input, output);
    }

    pub fn iter_connections(&self) -> impl Iterator<Item = (InputId, OutputId)> + '_ {
        self.connections.iter().map(|(o, i)| (o, *i))
    }

    pub fn connection(&self, input: InputId) -> Option<OutputId> {
        self.connections.get(input).copied()
    }

    pub fn any_param_type(&self, param: AnyParameterId) -> Result<&DataType, EguiGraphError> {
        match param {
            AnyParameterId::Input(input) => self.inputs.get(input).map(|x| &x.typ),
            AnyParameterId::Output(output) => self.outputs.get(output).map(|x| &x.typ),
        }
        .ok_or(EguiGraphError::InvalidParameterId(param))
    }

    pub fn try_get_input(
        &self,
        input: InputId,
    ) -> Option<&InputParam<DataType, ValueType, UserState>> {
        self.inputs.get(input)
    }

    pub fn get_input(&self, input: InputId) -> &InputParam<DataType, ValueType, UserState> {
        &self.inputs[input]
    }

    pub fn try_get_output(&self, output: OutputId) -> Option<&OutputParam<DataType, UserState>> {
        self.outputs.get(output)
    }

    pub fn get_output(&self, output: OutputId) -> &OutputParam<DataType, UserState> {
        &self.outputs[output]
    }
}

impl<
        NodeData: NodeDataTrait,
        DataType: DataTypeTrait<UserState>,
        ValueType: WidgetValueTrait,
        UserState: UserStateTrait,
    > Default for Graph<NodeData, DataType, ValueType, UserState>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<NodeData: NodeDataTrait> Node<NodeData> {
    pub fn inputs<
        'a,
        DataType: DataTypeTrait<UserState>,
        DataValue: WidgetValueTrait,
        UserState: UserStateTrait,
    >(
        &'a self,
        graph: &'a Graph<NodeData, DataType, DataValue, UserState>,
    ) -> impl Iterator<Item = &InputParam<DataType, DataValue, UserState>> + 'a {
        self.input_ids().map(|id| graph.get_input(id))
    }

    pub fn outputs<
        'a,
        DataType: DataTypeTrait<UserState>,
        DataValue: WidgetValueTrait,
        UserState: UserStateTrait,
    >(
        &'a self,
        graph: &'a Graph<NodeData, DataType, DataValue, UserState>,
    ) -> impl Iterator<Item = &OutputParam<DataType, UserState>> + 'a {
        self.output_ids().map(|id| graph.get_output(id))
    }

    pub fn input_ids(&self) -> impl Iterator<Item = InputId> + '_ {
        self.inputs.iter().map(|(_name, id)| *id)
    }

    pub fn output_ids(&self) -> impl Iterator<Item = OutputId> + '_ {
        self.outputs.iter().map(|(_name, id)| *id)
    }

    pub fn get_input(&self, name: &str) -> Result<InputId, EguiGraphError> {
        self.inputs
            .iter()
            .find(|(param_name, _id)| param_name == name)
            .map(|x| x.1)
            .ok_or_else(|| EguiGraphError::NoParameterNamed(self.id, name.into()))
    }

    pub fn get_output(&self, name: &str) -> Result<OutputId, EguiGraphError> {
        self.outputs
            .iter()
            .find(|(param_name, _id)| param_name == name)
            .map(|x| x.1)
            .ok_or_else(|| EguiGraphError::NoParameterNamed(self.id, name.into()))
    }
}
