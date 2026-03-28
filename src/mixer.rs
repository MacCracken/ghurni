//! Multi-component mechanical sound mixer.
//!
//! Combines multiple mechanical synthesizers into a single output
//! with independent gain and pan per component.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::traits::Synthesizer;

/// A named component in the mixer with gain and pan.
#[derive(Debug, Serialize, Deserialize)]
pub struct MixerChannel {
    /// Channel name (e.g., "engine", "exhaust", "turbo").
    pub name: String,
    /// Linear gain (0.0 = silent, 1.0 = unity).
    pub gain: f32,
    /// Stereo pan (-1.0 = full left, 0.0 = center, 1.0 = full right).
    pub pan: f32,
    /// Mute flag.
    pub muted: bool,
    /// The synthesizer for this channel.
    #[serde(skip)]
    synth: Option<Box<dyn Synthesizer>>,
    /// Scratch buffer for per-channel processing.
    #[serde(skip)]
    scratch: Vec<f32>,
}

impl MixerChannel {
    fn new(name: String, synth: Box<dyn Synthesizer>) -> Self {
        Self {
            name,
            gain: 1.0,
            pan: 0.0,
            muted: false,
            synth: Some(synth),
            scratch: Vec::new(),
        }
    }
}

/// Multi-component mechanical sound mixer.
///
/// Owns multiple `Synthesizer` instances and mixes them with
/// independent gain, pan, and mute per channel.
#[derive(Debug, Serialize, Deserialize)]
pub struct MechanicalMixer {
    channels: Vec<MixerChannel>,
    master_gain: f32,
}

impl MechanicalMixer {
    /// Creates a new empty mixer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            master_gain: 1.0,
        }
    }

    /// Adds a synthesizer channel. Returns the channel index.
    pub fn add_channel(&mut self, name: String, synth: Box<dyn Synthesizer>) -> usize {
        let idx = self.channels.len();
        self.channels.push(MixerChannel::new(name, synth));
        idx
    }

    /// Sets gain on a channel (0.0 = silent, 1.0 = unity).
    pub fn set_channel_gain(&mut self, index: usize, gain: f32) {
        if let Some(ch) = self.channels.get_mut(index) {
            ch.gain = gain.max(0.0);
        }
    }

    /// Sets pan on a channel (-1.0 left, 0.0 center, 1.0 right).
    pub fn set_channel_pan(&mut self, index: usize, pan: f32) {
        if let Some(ch) = self.channels.get_mut(index) {
            ch.pan = pan.clamp(-1.0, 1.0);
        }
    }

    /// Mutes or unmutes a channel.
    pub fn set_channel_muted(&mut self, index: usize, muted: bool) {
        if let Some(ch) = self.channels.get_mut(index) {
            ch.muted = muted;
        }
    }

    /// Sets the master output gain.
    pub fn set_master_gain(&mut self, gain: f32) {
        self.master_gain = gain.max(0.0);
    }

    /// Sets RPM on all channels.
    pub fn set_rpm(&mut self, rpm: f32) {
        for ch in &mut self.channels {
            if let Some(synth) = &mut ch.synth {
                synth.set_rpm(rpm);
            }
        }
    }

    /// Returns the number of channels.
    #[must_use]
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Processes all channels and mixes into a mono output.
    pub fn process_block(&mut self, output: &mut [f32]) {
        let len = output.len();
        for s in output.iter_mut() {
            *s = 0.0;
        }

        for ch in &mut self.channels {
            if ch.muted {
                continue;
            }
            if let Some(synth) = &mut ch.synth {
                // Ensure scratch buffer is large enough
                if ch.scratch.len() < len {
                    ch.scratch.resize(len, 0.0);
                }
                let scratch = &mut ch.scratch[..len];
                for s in scratch.iter_mut() {
                    *s = 0.0;
                }
                synth.process_block(scratch);

                let gain = ch.gain * self.master_gain;
                for (out, &src) in output.iter_mut().zip(scratch.iter()) {
                    *out += src * gain;
                }
            }
        }
    }

    /// Processes all channels into stereo output with panning.
    pub fn process_block_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        let len = left.len().min(right.len());
        for s in left[..len].iter_mut() {
            *s = 0.0;
        }
        for s in right[..len].iter_mut() {
            *s = 0.0;
        }

        for ch in &mut self.channels {
            if ch.muted {
                continue;
            }
            if let Some(synth) = &mut ch.synth {
                if ch.scratch.len() < len {
                    ch.scratch.resize(len, 0.0);
                }
                let scratch = &mut ch.scratch[..len];
                for s in scratch.iter_mut() {
                    *s = 0.0;
                }
                synth.process_block(scratch);

                // Equal-power pan law
                let pan_norm = (ch.pan + 1.0) * 0.5; // 0..1
                let angle = pan_norm * core::f32::consts::FRAC_PI_2;
                let (pan_sin, pan_cos) = (libm::sinf(angle), libm::cosf(angle));
                let gain_l = pan_cos * ch.gain * self.master_gain;
                let gain_r = pan_sin * ch.gain * self.master_gain;

                for i in 0..len {
                    left[i] += scratch[i] * gain_l;
                    right[i] += scratch[i] * gain_r;
                }
            }
        }
    }
}

impl Default for MechanicalMixer {
    fn default() -> Self {
        Self::new()
    }
}
