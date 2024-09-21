use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use serde::Deserialize;
use std::{os::unix::process::CommandExt, process::Command};

#[derive(Deserialize)]
pub struct PactlEntry {
    name: String,
    description: String,
}

pub struct State {
    outputs: Vec<PactlEntry>,
    inputs: Vec<PactlEntry>,
}

#[init]
fn init(_config_dir: RString) -> State {
    let pactl_inputs_raw = Command::new("bash").args(["-c", "pactl --format=json list sources | jq 'map(select(.name != \"easyeffects_source\" and (.description | startswith(\"Monitor\") | not)) | {name,description})'"]).output().expect("Failed to get sources");
    let pactl_inputs_raw_string =
        String::from_utf8(pactl_inputs_raw.stdout).expect("Failed to get sources stdout");
    let pactl_inputs: Vec<PactlEntry> =
        serde_json::from_str(&pactl_inputs_raw_string).expect("Failed to parse sources");

    let pactl_outputs_raw = Command::new("bash").args(["-c", "pactl --format=json list sinks | jq 'map(select(.name != \"easyeffects_sink\") | {name,description})'"]).output().expect("Failed to get sources");
    let pactl_outputs_raw_string =
        String::from_utf8(pactl_outputs_raw.stdout).expect("Failed to get sources stdout");
    let pactl_outputs: Vec<PactlEntry> =
        serde_json::from_str(&pactl_outputs_raw_string).expect("Failed to parse sources");

    State {
        inputs: pactl_inputs,
        outputs: pactl_outputs,
    }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "EasyEffects chooser".into(),
        icon: "audio-volume-medium".into(), // Icon from the icon theme
    }
}

#[get_matches]
fn get_matches(input: RString, state: &State) -> RVec<Match> {
    let mut inputs = state
        .inputs
        .iter()
        .filter_map(|entry| {
            if entry.description.contains(&input.to_string()) {
                Some(Match {
                    title: format!("Input: {}", entry.description).into(),
                    icon: ROption::RSome("audio-input-microphone-medium".into()),
                    use_pango: false,
                    description: ROption::RSome(entry.name.clone().into()),
                    id: ROption::RNone,
                })
            } else {
                None
            }
        })
        .collect::<Vec<Match>>();
    let mut outputs = state
        .outputs
        .iter()
        .filter_map(|entry| {
            if entry.description.contains(&input.to_string()) {
                Some(Match {
                    title: format!("Output: {}", entry.description).into(),
                    icon: ROption::RSome("audio-volume-medium".into()),
                    use_pango: false,
                    description: ROption::RSome(entry.name.clone().into()),
                    id: ROption::RNone,
                })
            } else {
                None
            }
        })
        .collect::<Vec<Match>>();
    let mut merged: Vec<Match> = Vec::new();
    merged.append(&mut outputs);
    merged.append(&mut inputs);

    RVec::from(merged)
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    let audio_type = match selection.title.chars().next() {
        Some('O') => "output",
        Some(_) => "input",
        None => "input",
    };
    let name = selection
        .description
        .expect("Error getting name")
        .to_string();
    let bypass = match name.as_str() {
        "alsa_output.usb-CHIYINFOX_Co._Ltd._GH510_000000000000-00.analog-stereo" => 2,
        _ => 1,
    };
    let command = format!(
        "gsettings set com.github.wwmm.easyeffects.stream{}s {}-device {} && easyeffects -b {}",
        audio_type, audio_type, name, bypass
    );
    Command::new("bash").args(["-c", &*command]).exec();
    HandleResult::Close
}
