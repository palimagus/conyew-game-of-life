
use rand::distributions::{Uniform, Distribution};
use wasm_bindgen::{JsValue, JsCast, prelude::Closure};
use yew::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, window};

const CELL_SIZE: u32 = 4;

pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<u8>,
}

impl Universe {
    pub fn new(width: u32, height: u32) -> Universe {
        let mut rng = rand::thread_rng();
        let die = Uniform::from(1..7);
        // Generate random cells
        let cells = (0..width * height)
            .map(|_| {
                let throw = die.sample(&mut rng);
                if throw <= 3 {
                    0
                } else {
                    1
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let mut live_neighbours = 0;

                for delta_row in [self.height - 1, 0, 1].iter().cloned() {
                    for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                        if delta_row == 0 && delta_col == 0 {
                            continue;
                        }

                        let neighbour_row = (row + delta_row) % self.height;
                        let neighbour_col = (col + delta_col) % self.width;
                        let idx = self.get_index(neighbour_row, neighbour_col);
                        live_neighbours += self.cells[idx];
                    }
                }

                next[idx] = match (cell, live_neighbours) {
                    (1, x) if x < 2 => 0,
                    (1, 2) | (1, 3) => 1,
                    (1, x) if x > 3 => 0,
                    (0, 3) => 1,
                    (otherwise, _) => otherwise,
                };
            }
        }

        self.cells = next;
    }
}

pub enum CanvasMessage {
    None, UniverseCreated, Render
}

struct AnimationCanvas {
    canvas: NodeRef,
    universe: Universe,
    callback: Closure<dyn FnMut()>,
}

impl AnimationCanvas {
    pub fn render(&mut self) {
        let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
        let ctx = canvas.get_context("2d").unwrap().unwrap();
        let ctx: CanvasRenderingContext2d = ctx.dyn_into().unwrap();

        ctx.set_fill_style(&JsValue::from_str("black"));
        ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        ctx.set_fill_style(&JsValue::from_str("white"));

        for row in 0..self.universe.height {
            for col in 0..self.universe.width {
                let idx = self.universe.get_index(row, col);
                if self.universe.cells[idx] == 0 {
                    continue;
                }

                ctx.fill_rect(
                    (col * CELL_SIZE) as f64,
                    (row * CELL_SIZE) as f64,
                    CELL_SIZE as f64,
                    CELL_SIZE as f64,
                );
            }
        }
        self.universe.tick();
        window().unwrap().request_animation_frame(
            self.callback.as_ref().unchecked_ref()).unwrap();
    }
}

impl Component for AnimationCanvas {
    type Message = CanvasMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            CanvasMessage::UniverseCreated
        });

        let comp_ctx = ctx.link().clone();
        let callback = Closure::wrap(Box::new(move || {
            comp_ctx.send_message(CanvasMessage::Render);
        }) as Box<dyn FnMut()>);

        Self {
            canvas: NodeRef::default(),
            universe: Universe::new(64, 64),
            callback,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CanvasMessage::None => false,
            CanvasMessage::UniverseCreated => {
                // Setup animation
                // Config canvas
                let width = self.universe.width * CELL_SIZE;
                let height = self.universe.height * CELL_SIZE;
                let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
                canvas.set_width(width);
                canvas.set_height(height);

                ctx.link().send_message(CanvasMessage::Render);

                true
            },
            CanvasMessage::Render => {
                self.render();
                false
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <canvas 
                id="canvas"
                ref={self.canvas.clone()}
            />
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
            <div class="canvas-title">
                <p class="title">
                    { "Game of Life" }
                </p>
                <p class="subtitle">
                    { "A Rust and WebAssembly implementation" }
                </p>
                <p class="author">
                    { "- Made with 🦀 and 🍵 by Anorak" }
                </p>
            </div>
            <div class="canvas-wrapper">
                <AnimationCanvas />
            </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}