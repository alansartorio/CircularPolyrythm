use itertools::Itertools;
use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use std::f64::consts::PI;
use std::iter;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    stream: audio::Stream<Audio>,
    balls: Vec<Ball>,
}

#[derive(Debug, Clone, Copy)]
struct Tone {
    vol: f64,
    hz: f64,
}

const MAX_TONES: usize = 10;

struct Audio {
    phase: f64,
    tones: [Tone; MAX_TONES],
}

struct Ball {
    angle: f64,
    speed: f64,
    tone: usize,
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window()
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    // Initialise the audio API so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // Initialise the state that we want to live on the audio thread.
    let model = Audio {
        phase: 0.0,
        tones: [Tone {
            hz: 440.0,
            vol: 1.0,
        }]
        .into_iter()
        .chain(iter::repeat(Tone {
            hz: 1000.0,
            vol: 0.0,
        }))
        .take(MAX_TONES)
        .collect_vec()
        .try_into()
        .unwrap(),
    };

    let stream = audio_host
        .new_output_stream(model)
        .render(audio)
        .build()
        .unwrap();

    stream.play().unwrap();

    //let mut scale = "WWHWWWH"
    let mut scale = "WHWWHWW"
        .chars()
        .map(|c| match c {
            'W' => 2,
            'H' => 1,
            _ => panic!(),
        })
        .cycle()
        .take(10)
        .scan(0, |acc, v| {
            *acc += v;
            Some(*acc)
        })
        .collect_vec();
    scale.insert(0, 0);
    Model {
        stream,
        balls: (0..10)
            .map(|i| Ball {
                angle: 0.0,
                speed: PI / 10.0 + PI * 2.0 / 40.0 * ((10 - i) as f64),
                tone: scale[i],
            })
            .collect_vec(),
    }
}

const SEMITONE: f64 = 1.0594630943592953;

// A function that renders the given `Audio` to the given `Buffer`.
// In this case we play a simple sine wave at the audio's current frequency in `hz`.
fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    let sample_rate = buffer.sample_rate() as f64;
    let volume = 0.5;
    for frame in buffer.frames_mut() {
        let sine_amp = audio
            .tones
            .map(|t| {
                (1..=3)
                    .map(|f| {
                        ((t.hz * audio.phase * (1.0 + (audio.phase * 7.0 * PI * 2.0).sin() * 0.000005) * (SEMITONE.powi((f - 1) * 7))) * PI).sin()
                            / (f as f64)
                    })
                    .into_iter()
                    .sum::<f64>()
                    * t.vol
            })
            .into_iter()
            .sum::<f64>() as f32;
        audio.phase += 1.0 / sample_rate;
        //audio.phase %= sample_rate;
        for tone in audio.tones.iter_mut() {
            tone.vol /= 1.00001;
        }
        for channel in frame {
            *channel = sine_amp * volume;
        }
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        // Pause or unpause the audio when Space is pressed.
        Key::Space => {
            //if model.stream.is_playing() {
            //model.stream.pause().unwrap();
            //} else {
            //model.stream.play().unwrap();
            //}
            model.stream.send(|audio| audio.tones[0].vol = 1.0).unwrap();
        }
        // Raise the frequency when the up key is pressed.
        Key::Up => {
            model
                .stream
                .send(|audio| {
                    audio.tones[0].hz *= SEMITONE.powi(7);
                })
                .unwrap();
        }
        // Lower the frequency when the down key is pressed.
        Key::Down => {
            model
                .stream
                .send(|audio| {
                    audio.tones[0].hz /= SEMITONE;
                })
                .unwrap();
        }
        _ => {}
    }
}

// Handle events related to the window and update the model if necessary
fn update(_app: &App, model: &mut Model, update: Update) {
    for (i, ball) in model.balls.iter_mut().enumerate() {
        ball.angle += ball.speed * update.since_last.as_secs_f64();
        if ball.angle > PI * 2.0 {
            let tone = ball.tone;
            model
                .stream
                .send(move |audio| {
                    audio.tones[i] = Tone {
                        hz: 440.0 * SEMITONE.powi(tone as i32),
                        vol: 1.0,
                    };
                })
                .unwrap();
            ball.angle -= PI * 2.0;
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let [w, h] = frame.texture_size();
    let draw = app.draw();
    draw.background().color(GRAY);
    draw.line()
        .points((0.0, 0.0).into(), (w as f32 / 2.0, 0.0).into())
        .color(DARKGRAY)
        .weight(3.0);
    for (i, ball) in model.balls.iter().enumerate() {
        let rad = (i + 1) as f32 * 30.0;
        draw.ellipse()
            .radius(rad)
            .stroke(LIGHTGRAY)
            .stroke_weight(3.0)
            .no_fill()
            .finish();
        draw.ellipse()
            .radius(10.0)
            .color(WHITE)
            .x_y(ball.angle.cos() as f32 * rad, ball.angle.sin() as f32 * rad)
            .finish();
    }

    draw.to_frame(app, &frame).unwrap();
}
