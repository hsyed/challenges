use crate::{game::Grid, patterns};
use gpui::*;
use gpui_component::{Disableable, button::Button};
use std::time::Duration;

pub struct GameOfLife {
    grid: Grid,
    next_grid: Grid,
    is_playing: bool,
    generation: usize,
    cell_size: f32,
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

    fn render_grid(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut rows = Vec::new();
        for y in 0..self.grid.height {
            let mut cells = Vec::new();
            for x in 0..self.grid.width {
                cells.push(self.render_cell(x, y, cx));
            }
            rows.push(div().flex().flex_row().gap_px().children(cells));
        }

        div()
            .flex()
            .flex_col()
            .gap_px()
            .bg(rgb(0x1e1e1e))
            .p_2()
            .children(rows)
    }

    fn render_cell(&self, x: usize, y: usize, cx: &mut Context<Self>) -> AnyElement {
        let alive = self.grid.get(x, y);
        let size = px(self.cell_size);

        div()
            .w(size)
            .h(size)
            .bg(if alive { rgb(0x00ff00) } else { rgb(0x333333) })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.toggle_cell(x, y, cx);
                }),
            )
            .cursor_pointer()
            .into_any_element()
    }
}

impl Render for GameOfLife {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .child(self.render_controls(cx))
            .child(self.render_grid(cx))
    }
}
