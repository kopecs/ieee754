#![warn(rust_2018_idioms)]
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use std::iter;

/// Number of **explicitly stored** significand bits for IEEE754 binary64.
const BINARY_64_SIGNIFICAND_BITS: usize = 52;

const BINARY_64_EXPONENT_BITS: usize = 11;

const BINARY_64_BIAS: usize = 1023;

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    let exponent_len = 11;
    let significand_len = 52;
    Model {
        sign_bit: false,
        exponent_bits: vec![false; exponent_len],
        significand_bits: vec![false; significand_len],
    }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
pub struct Model {
    sign_bit: bool,
    exponent_bits: Vec<bool>,
    significand_bits: Vec<bool>,
}

impl Model {
    // Move out to other struct if we end up storing more data than just number in Model
    fn value(&self) -> f64 {
        match (
            self.exponent_bits.iter().all(|&b| b),
            self.exponent_bits.iter().any(|&b| b),
        ) {
            // Special
            (true, _) => {
                if self.significand_bits.iter().any(|&b| b) {
                    f64::NAN
                } else if self.sign_bit {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                }
            }
            (false, normal) => {
                let bias: u64 = (1 << (self.exponent_bits.len() - 1)) - 1;
                let exp: u64 = self
                    .exponent_bits
                    .iter()
                    .fold(0, |acc, &b| (acc << 1) | (if b { 1 } else { 0 }));
                let significand: u64 = self
                    .significand_bits
                    .iter()
                    .fold(0, |acc, &b| (acc << 1) | (if b { 1 } else { 0 }));
                let sign = if self.sign_bit { 1 } else { 0 };
                f64::from_bits(
                    sign << (BINARY_64_EXPONENT_BITS + BINARY_64_SIGNIFICAND_BITS)
                        | if normal {
                            (exp + (BINARY_64_BIAS as u64 - bias)) << BINARY_64_SIGNIFICAND_BITS
                        } else {
                            0
                        }
                        | significand << BINARY_64_SIGNIFICAND_BITS - self.significand_bits.len(),
                )
            }
        }
    }
}

// For some styling later
#[derive(Debug, Copy, Clone)]
enum BitType {
    Sign,
    Exponent,
    Significand,
}

impl BitType {
    fn color(&self) -> &'static str {
        match self {
            Self::Sign => "#D72638",
            Self::Exponent => "#00916E",
            Self::Significand => "#F49D37",
        }
    }
}

// ------ ------
//    Update
// ------ ------

// `Msg` describes the different events you can modify state with.
#[derive(Copy, Clone)]
enum Msg {
    SetExpSize(usize),
    SetSigSize(usize),
    ToggleBit(usize),
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::SetExpSize(e) => model.exponent_bits.resize(e, false),
        Msg::SetSigSize(s) => model.significand_bits.resize(s, false),
        Msg::ToggleBit(b) => {
            if let Some(bit) = iter::once(&mut model.sign_bit)
                .chain(&mut model.exponent_bits)
                .chain(&mut model.significand_bits)
                .nth(b)
            {
                *bit = !*bit;
            }
        }
    }
}

// ------ ------
//     View
// ------ ------

// `view` describes what to display.
fn view(model: &Model) -> Vec<Node<Msg>> {
    nodes![
        view_value(model),
        view_bits(model),
        div![
            C!["controls"],
            div![
                C!["exponent_slider"],
                format!(
                    "Exponent Bits ({}): ",
                    model.exponent_bits.len().to_string()
                ),
                input![
                    attrs! {
                        At::Type => "range",
                        At::Min => "1",
                        At::Max => BINARY_64_EXPONENT_BITS.to_string(),
                        At::Value => model.exponent_bits.len().to_string()
                    },
                    input_ev(Ev::Input, |i| Msg::SetExpSize(
                        i.parse().expect("Slider must report number")
                    )),
                ],
            ],
            div![
                C!["significand_slider"],
                format!(
                    "Significand Bits ({}): ",
                    model.significand_bits.len().to_string()
                ),
                input![
                    attrs! {
                        At::Type => "range",
                        At::Min => "1",
                        At::Max => BINARY_64_SIGNIFICAND_BITS.to_string(),
                        At::Value => model.significand_bits.len().to_string()
                    },
                    input_ev(Ev::Input, |i| Msg::SetSigSize(
                        i.parse().expect("Slider must report number")
                    )),
                ],
            ],
        ]
    ]
}

fn view_bits(model: &Model) -> Node<Msg> {
    div![
        C!["bits"],
        iter::once(&model.sign_bit)
            .zip(iter::repeat(BitType::Sign))
            .chain(
                model
                    .exponent_bits
                    .iter()
                    .zip(iter::repeat(BitType::Exponent))
            )
            .chain(
                model
                    .significand_bits
                    .iter()
                    .zip(iter::repeat(BitType::Significand))
            )
            .enumerate()
            .map(|(i, (&b, t))| button![
                C!["bit"],
                style! {St::BackgroundColor => t.color() },
                if b { "1" } else { "0" },
                ev(Ev::Click, move |_| Msg::ToggleBit(i))
            ]),
    ]
}

fn view_value(model: &Model) -> Node<Msg> {
    div![id!["result"], C!["value"], {
        let value = model.value();
        let abs_val = value.abs();
        if abs_val == 0.0 || (1.0e-10..1.0e10).contains(&abs_val) {
            format!("{:?}", value)
        } else {
            format!("{:e}", value)
        }
    }]
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
