use itertools::Itertools;
use nannou::draw::primitive::text::Style;
use nannou::{prelude::*, rand};
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
    phase: f64,
}

const MAX_TONES: usize = 30;

struct Audio {
    tones: [Tone; MAX_TONES],
}

struct Ball {
    angle: f64,
    speed: f64,
    tone: i64,
}

fn generate_tones(start: i64, intervals: Vec<usize>) -> impl Iterator<Item = i64> {
    intervals.into_iter().cycle().scan(start, |acc, v| {
        let val = *acc;
        *acc += v as i64;
        Some(val)
    })
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
        tones: iter::repeat(Tone {
            hz: 1000.0,
            vol: 0.0,
            phase: 0.0,
        })
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

    //let scale = "WHWWHWW"
    //let scale = "WWHWWWH"
    //let scale = "H"
    //let scale = "7543412212221"
    //let scale = "7498732" // 10 notas
    //let scale = "28781512" // 10 notas
    let scale = "439829";
    let scale = generate_tones(
        0,
        scale
            .chars()
            .map(|c| match c {
                _ if c.is_digit(16) => c.to_digit(16).unwrap() as usize,
                _ => panic!(),
            })
            .collect_vec(),
    );
    let ball_count = 7;
    Model {
        stream,
        balls: (0..ball_count)
            .zip(scale)
            .map(|(i, tone)| Ball {
                angle: 0.0,
                speed: PI * 2.0 / 40.0 + PI * 2.0 * PI / 3.0 / 80.0 * ((ball_count - i) as f64),
                tone: tone as i64 - 12,
            })
            .collect_vec(),
    }
}

const SEMITONE: f64 = 1.0594630943592953;

// A function that renders the given `Audio` to the given `Buffer`.
// In this case we play a simple sine wave at the audio's current frequency in `hz`.
fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    let fm_freq = 5.0;
    let fm_amp = 0.000005;
    //let fm_amp = 0.001;
    let sample_rate = buffer.sample_rate() as f64;
    let volume = 0.5;
    for frame in buffer.frames_mut() {
        let sine_amp = audio
            .tones
            .map(|t| {
                (1..=2)
                    .map(|f| {
                        ((t.hz
                            * t.phase
                            * (1.0 + (t.phase * fm_freq * PI * 2.0).sin() * fm_amp)
                            * (1.5.powi(f - 1)))
                            * PI)
                            .sin()
                            / (f as f64)
                    })
                    .into_iter()
                    .sum::<f64>()
                    * (1.0 + (t.phase * 3.0 * PI * 2.0).sin() * 0.3)
                    * t.vol
            })
            .into_iter()
            .sum::<f64>() as f32;
        //audio.phase %= sample_rate;
        for tone in audio.tones.iter_mut() {
            tone.phase += 1.0 / sample_rate;
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
            let starting_note = rand::random_range(-16, 16);
            let notes = generate_tones(starting_note, (0..10).map(|_| rand::random_range(1, 10)).collect_vec());
            let ball_count = rand::random_range(6, 12);
            model.balls = (0..ball_count)
                .zip(notes)
                .map(|(i, tone)| Ball {
                    angle: 2.0 * PI - 0.3,
                    speed: PI * 2.0 / 40.0 + PI * 2.0 * PI / 3.0 / 80.0 * ((ball_count - i) as f64),
                    tone: tone as i64 - 12,
                })
                .collect_vec();
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
                        vol: 0.1,
                        phase: 0.0,
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
        let ball_translation = draw.translate(Vec3::new(
            ball.angle.cos() as f32 * rad,
            ball.angle.sin() as f32 * rad,
            0.0,
        ));
        ball_translation
            .ellipse()
            .radius(10.0)
            .color(WHITE)
            .finish();
        ball_translation
            .text(
                [
                    "A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#",
                ][ball.tone.rem_euclid(12) as usize],
            )
            .color(BLACK)
            .align_text_middle_y()
            .finish()
    }

    draw.to_frame(app, &frame).unwrap();
}
