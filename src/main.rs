use nannou::prelude::*;
use rand::Rng;
use serde_json;
use std::cmp::{max, min};
use std::sync::mpsc::{channel, Receiver};
use websocket::client::sync::Client;
use websocket::message::OwnedMessage;
use websocket::sync::stream::TcpStream;
use websocket::{ClientBuilder, Message};

mod messages;

use messages::Messages;

fn main() {
    nannou::app(model).update(update).run()
}

struct Model {
    // Store the window ID so we can refer to this specific window later if needed.
    _window: WindowId,
    matrix: Vec<i32>,
    buffers: Vec<Vec<i32>>,
    matrix_position: usize,
    skipped: bool,
    num_steps_on_screen: usize,
    tempo: f32,
    num_graphs: usize,
    ws_client: websocket::sender::Writer<TcpStream>,
    ws_receiver: Receiver<Messages>,
}

impl Model {
    pub fn increment_num_steps_on_screen(&mut self) {
        for b in self.buffers.iter_mut() {
            b.insert(0, 0);
        }
        self.num_steps_on_screen = min(self.num_steps_on_screen + 1, 64);
    }

    pub fn decrement_num_steps_on_screen(&mut self) {
        if self.num_steps_on_screen > 16 {
            for b in self.buffers.iter_mut() {
                b.remove(0);
            }
        }
        self.num_steps_on_screen = max(self.num_steps_on_screen - 1, 16);
    }
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let _window = app
        .new_window()
        .size(512, 512)
        .title("nannou")
        .view(view) // The function that will be called for presenting graphics to a frame.
        .event(event) // The function that will be called when the window receives events.
        .build()
        .unwrap();

    let num_steps_on_screen = 32;
    app.set_loop_mode(LoopMode::RefreshSync);
    let matrix = vec![0; 64];
    let buffers: Vec<Vec<i32>> = vec![vec![0; num_steps_on_screen + 1]; 4];
    let ws_client = ClientBuilder::new("ws://127.0.0.1:8080")
        .unwrap()
        .connect_insecure()
        .unwrap();

    let (mut receiver, sender) = ws_client.split().unwrap();
    let (send, recv): (_, Receiver<Messages>) = channel();

    let model = Model {
        _window,
        matrix,
        buffers,
        matrix_position: 0,
        skipped: true,
        num_steps_on_screen,
        tempo: 20.0,
        num_graphs: 4,
        ws_client: sender,
        ws_receiver: recv,
    };

    std::thread::spawn(move || {
        for message in receiver.incoming_messages() {
            if let Some(m) = message.ok() {
                println!("Recv: {:?}", m);
                match m {
                    OwnedMessage::Text(msg) => {
                        let maybe_server_msg: Option<messages::ServerMessage> =
                            serde_json::from_str(&msg).ok();
                        if let Some(server_msg) = maybe_server_msg {
                            if server_msg.addr == "/matrix" {
                                let internal_msg: Option<messages::MatrixMessage> =
                                    serde_json::from_str(&msg).ok();
                                if let Some(internal_msg) = internal_msg {
                                    send.send(Messages::Matrix(internal_msg)).unwrap();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });
    model
}

// Handle events related to the window and update the model if necessary
fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        // generate random matrix on mouse press
        // WindowEvent::MousePressed(_) => {
        //     let mut rng = rand::thread_rng();
        //     model.matrix = model.matrix.iter().map(|_| rng.gen_range(0..2)).collect();
        //     dbg!(&model.matrix);
        // }
        WindowEvent::KeyPressed(key) => match key {
            Key::Left => {
                dbg!("left");
                model.increment_num_steps_on_screen();
            }
            Key::Right => {
                dbg!("right");
                model.decrement_num_steps_on_screen();
            }
            Key::Key1 => {
                model.num_graphs = 1;
            }
            Key::Key2 => {
                model.num_graphs = 2;
            }
            Key::Key3 => {
                model.num_graphs = 3;
            }
            Key::Key4 => {
                model.num_graphs = 4;
            }
            Key::S => {
                let matrix_request = messages::MatrixRequestMessage::new();
                let mr_json = serde_json::to_string(&matrix_request).unwrap();
                let m = Message::text(&mr_json);
                model.ws_client.send_message(&m).unwrap();
            }
            _ => (),
        },
        _ => (),
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let win = app.window_rect();
    let step_size = win.w() / model.num_steps_on_screen as f32;
    let t = app.duration.since_start.as_secs_f32();
    let tempo = model.tempo;
    let offset = (t * tempo) % step_size;
    if offset < 1.0 && !model.skipped {
        let matrix_cycle_len = model.matrix.len() / model.num_graphs;
        model.matrix_position = (model.matrix_position + 1) % matrix_cycle_len;
        for (i, b) in model.buffers.iter_mut().enumerate() {
            if i < model.num_graphs {
                b.remove(0);
                b.push(model.matrix[model.matrix_position + (i * matrix_cycle_len)]);
            }
        }
        model.skipped = true
    } else if offset >= 1.0 {
        model.skipped = false
    }

    if let Some(m) = model.ws_receiver.try_recv().ok() {
        match m {
            Messages::Matrix(m) => {
                model.matrix = m.matrix;
            }
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();

    // Clear the background to black.
    draw.background().color(BLACK);

    let win = app.window_rect();
    let step_size = win.w() / model.num_steps_on_screen as f32;
    let t = app.duration.since_start.as_secs_f32();

    let rect_height = win.h() * 0.1;
    let mut prev = 0;
    let tempo = model.tempo;
    let offset = (t * tempo) % step_size;
    let offset = -1.0 * offset;
    let line_weight = 2.0;
    let x_offset = win.w() * -0.5;

    // Draw the line!
    for (n, b) in model.buffers.iter().enumerate() {
        if n < model.num_graphs {
            let y_baseline = (win.h() * 0.35) - (win.h() * 0.25 * n as f32);
            for (i, v) in b.iter().enumerate() {
                if *v == 1 {
                    if prev == 0 {
                        draw.line().weight(line_weight).color(GREEN).points(
                            geom::point::pt2(
                                x_offset + offset + (step_size * i as f32),
                                y_baseline,
                            ),
                            geom::point::pt2(
                                x_offset + offset + (step_size * i as f32),
                                y_baseline + rect_height + (line_weight * 0.5),
                            ),
                        );
                    }
                    draw.line().weight(line_weight).color(GREEN).points(
                        geom::point::pt2(
                            x_offset + offset + (step_size * i as f32),
                            y_baseline + rect_height,
                        ),
                        geom::point::pt2(
                            x_offset + offset + (step_size * (i + 1) as f32),
                            y_baseline + rect_height,
                        ),
                    );
                } else {
                    if prev == 1 {
                        draw.line().weight(line_weight).color(GREEN).points(
                            geom::point::pt2(
                                x_offset + offset + (step_size * i as f32),
                                y_baseline + rect_height + (line_weight * 0.5),
                            ),
                            geom::point::pt2(
                                x_offset + offset + (step_size * i as f32),
                                y_baseline + line_weight * -0.5,
                            ),
                        );
                    }
                    draw.line().weight(line_weight).color(GREEN).points(
                        geom::point::pt2(x_offset + offset + (step_size * i as f32), y_baseline),
                        geom::point::pt2(
                            x_offset + offset + (step_size * (i + 1) as f32),
                            y_baseline,
                        ),
                    );
                }
                prev = *v;
            }
        }

        let future_width = win.w() * 0.2;
        let future_x_position = win.right() - future_width;

        // cover the right edge of the lines with an opaque rectangle
        draw.rect()
            .w_h(future_width, win.h())
            .x_y(future_x_position + (future_width * 0.5), 0.0)
            .rgba(0.0, 0.0, 0.0, 0.3);

        // mark the moment with a dashed line
        let num_dashes = 64;
        let dash_length = win.h() / (num_dashes * 2) as f32;

        for i in 0..num_dashes {
            draw.line().color(GREEN).weight(1.0).points(
                geom::point::pt2(
                    future_x_position,
                    win.h() * 0.5 - (i as f32 * dash_length * 2.0),
                ),
                geom::point::pt2(
                    future_x_position,
                    win.h() * 0.5 - (i as f32 * dash_length * 2.0) - dash_length,
                ),
            );
        }
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
