use eframe::egui::{self, ScrollArea, TextStyle};

pub struct IndexListComponent<I: Copy> {
    index_list: Vec<(String, I)>,
    selected_row: usize,
    list_ordering: IndexListOrdering,
    delayed_row_selection: Option<DelayedRowSelection>,
}

pub enum IndexListOrdering {
    /// Doesn't respect any particular order
    None,
    /// Orders types alphabetically
    Alphabetical,
}

#[derive(Clone)]
struct DelayedRowSelection {
    align: Option<egui::Align>,
    request_focus: bool,
}

impl<I: Copy> IndexListComponent<I> {
    pub fn new(ordering: IndexListOrdering) -> Self {
        Self {
            index_list: vec![],
            selected_row: usize::MAX,
            list_ordering: ordering,
            delayed_row_selection: None,
        }
    }

    pub fn update_index_list(&mut self, index_list: Vec<(String, I)>) {
        self.index_list = index_list;
        self.selected_row = usize::MAX;

        // Reorder list if needed
        if let IndexListOrdering::Alphabetical = self.list_ordering {
            self.index_list
                .sort_unstable_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
        }
    }

    pub fn update<CB: FnMut(&str, I)>(&mut self, ui: &mut egui::Ui, on_element_selected: &mut CB) {
        let num_rows = self.index_list.len();
        const TEXT_STYLE: TextStyle = TextStyle::Body;
        let row_height = ui.text_style_height(&TEXT_STYLE);
        ui.with_layout(
            egui::Layout::top_down(egui::Align::Min).with_cross_justify(true),
            |ui| {
                if num_rows == 0 {
                    // Display a default message to make it obvious the list is empty
                    ui.label("No results");
                    return;
                }

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show_rows(ui, row_height, num_rows, |ui, row_range| {
                        for row_index in row_range.clone() {
                            let (type_name, type_index) = &self.index_list[row_index];

                            let label =
                                ui.selectable_label(self.selected_row == row_index, type_name);

                            // If label was clicked this frame, select the corresponding element
                            if label.clicked() {
                                self.selected_row = row_index;
                                on_element_selected(type_name, *type_index);

                                // Set keyboard focus on the widget, to enable keyboard navigation
                                label.request_focus();
                            }
                            // Else if label was selected via keyboard navigation on the previous frame, select the corresponding element
                            else if let Some(delayed_row_selection) =
                                self.delayed_row_selection.clone()
                            {
                                if self.selected_row == row_index {
                                    self.delayed_row_selection = None;
                                    on_element_selected(type_name, *type_index);

                                    // Scroll to the label, in case we jumped to a row which is far from the previously selected one
                                    label.scroll_to_me(delayed_row_selection.align);
                                    if delayed_row_selection.request_focus {
                                        // Set keyboard focus on the widget, to enable keyboard navigation
                                        label.request_focus();
                                    }
                                }
                            }

                            // If label has keyboard focus, handle keyboard navigation
                            if ui.memory(|m| m.has_focus(label.id)) {
                                // Arrow up/down -> select previous/next label
                                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                                    // Only handle this if we're within range
                                    if self.selected_row > 0 {
                                        self.selected_row -= 1;
                                        self.delayed_row_selection = Some(DelayedRowSelection {
                                            align: None,
                                            request_focus: false,
                                        });
                                    }
                                } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                                    // Only handle this if we're within range
                                    if self.selected_row < num_rows - 1 {
                                        self.selected_row += 1;
                                        self.delayed_row_selection = Some(DelayedRowSelection {
                                            align: None,
                                            request_focus: false,
                                        });
                                    }
                                }
                                // Page up/down -> scroll up/down in the list
                                else if ui.input(|i| i.key_pressed(egui::Key::PageUp)) {
                                    self.selected_row = row_range.start;
                                    self.delayed_row_selection = Some(DelayedRowSelection {
                                        align: Some(egui::Align::Center),
                                        request_focus: true,
                                    });
                                } else if ui.input(|i| i.key_pressed(egui::Key::PageDown)) {
                                    self.selected_row = row_range.end - 1;
                                    self.delayed_row_selection = Some(DelayedRowSelection {
                                        align: Some(egui::Align::Center),
                                        request_focus: true,
                                    });
                                }
                            }
                        }
                    });
            },
        );
    }
}

impl<I: Copy> Default for IndexListComponent<I> {
    fn default() -> Self {
        Self::new(IndexListOrdering::None)
    }
}
