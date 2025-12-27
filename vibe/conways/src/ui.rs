use crate::{game::Grid, patterns};
use gpui::*;
use gpui_component::{Disableable, button::Button};
use std::time::Duration;

const AVAILABLE_PATTERNS: &[(&str, fn(&mut Grid, usize, usize))] = &[
    ("Glider", patterns::load_glider),
    ("Blinker", patterns::load_blinker),
    ("Toad", patterns::load_toad),
    ("Beacon", patterns::load_beacon),
    ("Pulsar", patterns::load_pulsar),
    ("LWSS", patterns::load_lwss),
    ("Pentadecathlon", patterns::load_pentadecathlon),
];

pub struct GameOfLife {
    grid: Grid,
    next_grid: Grid,
    is_playing: bool,
    generation: usize,
    cell_size: f32,
    pattern_picker_open: bool,
    pattern_picker_position: (usize, usize),
    preview_position: Option<(usize, usize)>,
    selected_pattern: Option<fn(&mut Grid, usize, usize)>,
}

impl GameOfLife {
    pub fn new(cx: &Context<Self>) -> Self {
        let mut grid = Grid::new(100, 100);
        let next_grid = Grid::new(100, 100);

        // Load demo pattern
        patterns::load_demo_scene(&mut grid);

        let mut game = Self {
            grid,
            next_grid,
            is_playing: false,
            generation: 0,
            cell_size: 6.0,
            pattern_picker_open: false,
            pattern_picker_position: (0, 0),
            preview_position: None,
            selected_pattern: None,
        };

        // Start the game loop
        game.start_game_loop(cx);

        game
    }

    fn start_game_loop(&mut self, cx: &Context<Self>) {
        cx.spawn(async |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;

                this.update(cx, |entity, cx| {
                    if entity.is_playing {
                        entity.step(cx);
                    }
                })
                .unwrap();
            }
        })
        .detach();
    }

    fn step(&mut self, cx: &mut Context<Self>) {
        self.next_grid = self.grid.next_generation();
        std::mem::swap(&mut self.grid, &mut self.next_grid);
        self.generation += 1;
        cx.notify();
    }

    fn toggle_playing(&mut self, cx: &mut Context<Self>) {
        self.is_playing = !self.is_playing;
        cx.notify();
    }

    fn reset(&mut self, cx: &mut Context<Self>) {
        self.grid.clear();
        patterns::load_demo_scene(&mut self.grid);
        self.generation = 0;
        cx.notify();
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        self.grid.clear();
        self.generation = 0;
        cx.notify();
    }

    fn toggle_cell(&mut self, x: usize, y: usize, cx: &mut Context<Self>) {
        if !self.is_playing {
            self.grid.toggle(x, y);
            cx.notify();
        }
    }

    fn open_pattern_picker(&mut self, x: usize, y: usize, cx: &mut Context<Self>) {
        if !self.is_playing {
            self.pattern_picker_position = (x, y);
            self.pattern_picker_open = true;
            cx.notify();
        }
    }

    fn close_pattern_picker(&mut self, cx: &mut Context<Self>) {
        self.pattern_picker_open = false;
        cx.notify();
    }

    fn select_pattern(
        &mut self,
        load_fn: fn(&mut Grid, usize, usize),
        cx: &mut Context<Self>,
    ) {
        self.selected_pattern = Some(load_fn);
        self.close_pattern_picker(cx);
    }

    fn place_pattern(&mut self, x: usize, y: usize, cx: &mut Context<Self>) {
        if let Some(load_fn) = self.selected_pattern {
            // Load pattern into template grid at fixed position to match preview
            let mut temp_grid = Grid::new(100, 100);
            load_fn(&mut temp_grid, 10, 10);
            
            // Copy the pattern from template to actual board at cursor position
            for ty in 0..100 {
                for tx in 0..100 {
                    if temp_grid.get(tx, ty) {
                        let board_x = (x as isize + (tx as isize - 10)) as usize;
                        let board_y = (y as isize + (ty as isize - 10)) as usize;
                        if board_x < self.grid.width && board_y < self.grid.height {
                            self.grid.set(board_x, board_y, true);
                        }
                    }
                }
            }
            
            self.selected_pattern = None;
            self.preview_position = None;
            cx.notify();
        }
    }

    fn cancel_pattern_placement(&mut self, cx: &mut Context<Self>) {
        self.selected_pattern = None;
        self.preview_position = None;
        cx.notify();
    }

    fn set_preview_position(&mut self, x: usize, y: usize, cx: &mut Context<Self>) {
        self.preview_position = Some((x, y));
        cx.notify();
    }

    fn clear_preview_position(&mut self, cx: &mut Context<Self>) {
        self.preview_position = None;
        cx.notify();
    }

    fn render_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_playing = self.is_playing;
        let generation = self.generation;

        div()
            .flex()
            .flex_row()
            .gap_2()
            .p_4()
            .items_center()
            .justify_center()
            .bg(rgb(0x2d2d2d))
            .child(
                Button::new("play_pause")
                    .label(if is_playing { "Pause" } else { "Play" })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.toggle_playing(cx);
                    })),
            )
            .child(
                Button::new("step")
                    .label("Step")
                    .disabled(is_playing)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.step(cx);
                    })),
            )
            .child(
                Button::new("reset")
                    .label("Reset")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.reset(cx);
                    })),
            )
            .child(
                Button::new("clear")
                    .label("Clear")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.clear(cx);
                    })),
            )
            .child(
                div()
                    .ml_4()
                    .text_color(rgb(0xcccccc))
                    .child(format!("Generation: {}", generation)),
            )
    }

    fn render_preview_overlay(&self) -> impl IntoElement {
        if let Some((prev_x, prev_y)) = self.preview_position {
            if let Some(pattern_fn) = self.selected_pattern {
                // Load pattern into a fixed position to avoid wrapping issues
                let mut temp_grid = Grid::new(100, 100);
                pattern_fn(&mut temp_grid, 10, 10);

                // Build preview for the visible area around cursor, using the pattern template
                let mut rows = Vec::new();
                let start_y = prev_y.saturating_sub(2);
                let end_y = (prev_y + 20).min(self.grid.height);
                let start_x = prev_x.saturating_sub(2);
                let end_x = (prev_x + 20).min(self.grid.width);
                
                for ty in start_y..end_y {
                    let mut cells = Vec::new();
                    
                    for tx in start_x..end_x {
                        // Map cursor position to the template grid offset
                        let template_x = (tx - prev_x) + 10;
                        let template_y = (ty - prev_y) + 10;
                        let alive = if template_x < 100 && template_y < 100 {
                            temp_grid.get(template_x, template_y)
                        } else {
                            false
                        };
                        
                        let size = px(self.cell_size);
                        
                        if alive {
                            cells.push(
                                div()
                                    .w(size)
                                    .h(size)
                                    .bg(rgb(0x0066ff))
                                    .opacity(0.5)
                                    .border_1()
                                    .border_color(rgb(0x0066ff))
                                    .into_any_element(),
                            );
                        } else {
                            cells.push(
                                div()
                                    .w(size)
                                    .h(size)
                                    .into_any_element(),
                            );
                        }
                    }
                    
                    // Show all rows in the visible window, even if empty
                    rows.push(div().flex().flex_row().gap_px().children(cells));
                }

                // Calculate absolute position based on first visible cell
                let offset_x = px(start_x as f32 * (self.cell_size + 1.0) + 8.0); // 8px is padding
                let offset_y = px(start_y as f32 * (self.cell_size + 1.0) + 8.0);

                div()
                    .absolute()
                    .left(offset_x)
                    .top(offset_y)
                    .flex()
                    .flex_col()
                    .gap_px()
                    .children(rows)
                    .into_any_element()
            } else {
                div().into_any_element()
            }
        } else {
            div().into_any_element()
        }
    }

    fn render_grid(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut rows = Vec::new();
        for y in 0..self.grid.height {
            let mut cells = Vec::new();
            for x in 0..self.grid.width {
                cells.push(self.render_cell(x, y, cx));
            }
            rows.push(div().flex().flex_row().gap_px().children(cells));
        }

        let mut grid = div()
            .flex()
            .flex_col()
            .gap_px()
            .bg(rgb(0x1e1e1e))
            .p_2()
            .relative()
            .children(rows);

        if self.selected_pattern.is_some() {
            grid = grid.child(self.render_preview_overlay());
        }

        grid
    }

    fn render_cell(&self, x: usize, y: usize, cx: &mut Context<Self>) -> AnyElement {
        let alive = self.grid.get(x, y);
        let size = px(self.cell_size);
        let picker_open = self.pattern_picker_open;
        let pattern_selected = self.selected_pattern.is_some();

        div()
            .w(size)
            .h(size)
            .bg(if alive { rgb(0x00ff00) } else { rgb(0x333333) })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    if pattern_selected {
                        // Place the selected pattern
                        this.place_pattern(x, y, cx);
                    } else if !picker_open {
                        this.toggle_cell(x, y, cx);
                    }
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, _, _, cx| {
                    if pattern_selected {
                        // Cancel pattern placement
                        this.cancel_pattern_placement(cx);
                    } else {
                        this.open_pattern_picker(x, y, cx);
                    }
                }),
            )
            .on_mouse_move(
                cx.listener(move |this, _event, _window, cx| {
                    if this.selected_pattern.is_some() {
                        this.set_preview_position(x, y, cx);
                    }
                }),
            )
            .cursor_pointer()
            .into_any_element()
    }

    fn render_pattern_preview(&self, load_fn: fn(&mut Grid, usize, usize)) -> impl IntoElement {
        // Load pattern into a full grid to avoid wrapping issues
        let mut preview_grid = Grid::new(100, 100);
        load_fn(&mut preview_grid, 10, 10);

        // Show a fixed 20x20 window around the pattern center
        let mut rows = Vec::new();
        for y in 0..20 {
            let mut cells = Vec::new();
            
            for x in 0..20 {
                let alive = preview_grid.get(x + 5, y + 5);
                cells.push(
                    div()
                        .w(px(4.0))
                        .h(px(4.0))
                        .bg(if alive { rgb(0x00ff00) } else { rgb(0x444444) })
                        .into_any_element(),
                );
            }
            
            // Show all rows to maintain consistent preview size
            rows.push(div().flex().flex_row().gap_px().children(cells));
        }

        div()
            .flex()
            .flex_col()
            .gap_px()
            .bg(rgb(0x1e1e1e))
            .p_1()
            .children(rows)
    }

    fn render_pattern_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut pattern_items = Vec::new();

        for (idx, (name, load_fn)) in AVAILABLE_PATTERNS.iter().enumerate() {
            let preview = self.render_pattern_preview(*load_fn);
            
            pattern_items.push(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .items_center()
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            let load_fn = AVAILABLE_PATTERNS[idx].1;
                            this.select_pattern(load_fn, cx);
                        }),
                    )
                    .child(preview)
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xcccccc))
                            .child(*name),
                    )
                    .into_any_element(),
            );
        }

        // Modal overlay with centered content
        div()
            .absolute()
            .inset_0()
            .bg(rgb(0x000000))
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.close_pattern_picker(cx);
                }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_6()
                    .bg(rgb(0x2d2d2d))
                    .rounded_lg()
                    .shadow_lg()
                    .max_w(px(900.0))
                    .child(
                        div()
                            .text_lg()
                            .text_color(rgb(0xcccccc))
                            .w_full()
                            .justify_center()
                            .child("Select Pattern"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_4()
                            .justify_center()
                            .children(pattern_items),
                    )
            )
    }
}

impl Render for GameOfLife {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut container = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .child(self.render_controls(cx))
            .child(self.render_grid(cx));

        if self.pattern_picker_open {
            container = container.child(self.render_pattern_picker(cx));
        }

        container
    }
}
