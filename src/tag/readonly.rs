use std::fmt;
use std::time::Duration;

use crate::{AudioInfo, ChannelConfig, SampleRate, Tag};

/// ### Audio information
impl Tag {
    /// Returns a reference of the audio information.
    pub fn audio_info(&self) -> &AudioInfo {
        &self.info
    }

    /// Returns the duration in seconds.
    pub fn duration(&self) -> Option<Duration> {
        self.info.duration
    }

    /// Returns the duration formatted in an easily readable way.
    pub(crate) fn format_duration(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let duration = match self.duration() {
            Some(d) => d,
            None => return Ok(()),
        };
        let total_seconds = duration.as_secs();
        let nanos = duration.subsec_nanos();
        let micros = nanos / 1_000;
        let millis = nanos / 1_000_000;
        let seconds = total_seconds % 60;
        let minutes = total_seconds / 60 % 60;
        let hours = total_seconds / 60 / 60;

        match (hours, minutes, seconds, millis, micros, nanos) {
            (0, 0, 0, 0, 0, n) => writeln!(f, "duration: {n}ns"),
            (0, 0, 0, 0, u, _) => writeln!(f, "duration: {u}µs"),
            (0, 0, 0, m, _, _) => writeln!(f, "duration: {m}ms"),
            (0, 0, s, _, _, _) => writeln!(f, "duration: {s}s"),
            (0, m, s, _, _, _) => writeln!(f, "duration: {m}:{s:02}"),
            (h, m, s, _, _, _) => writeln!(f, "duration: {h}:{m:02}:{s:02}"),
        }
    }

    /// Returns the channel configuration.
    pub fn channel_config(&self) -> Option<ChannelConfig> {
        self.info.channel_config
    }

    pub(crate) fn format_channel_config(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.channel_config() {
            Some(c) => writeln!(f, "channel config: {c}"),
            None => Ok(()),
        }
    }

    /// Returns the channel configuration.
    pub fn sample_rate(&self) -> Option<SampleRate> {
        self.info.sample_rate
    }

    pub(crate) fn format_sample_rate(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.sample_rate() {
            Some(r) => writeln!(f, "sample rate: {r}"),
            None => Ok(()),
        }
    }

    /// Returns the average bitrate.
    pub fn avg_bitrate(&self) -> Option<u32> {
        self.info.avg_bitrate
    }

    pub(crate) fn format_avg_bitrate(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.avg_bitrate() {
            Some(c) => writeln!(f, "average bitrate: {}kbps", c / 1024),
            None => Ok(()),
        }
    }

    /// Returns the maximum bitrate.
    pub fn max_bitrate(&self) -> Option<u32> {
        self.info.max_bitrate
    }

    pub(crate) fn format_max_bitrate(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.max_bitrate() {
            Some(c) => writeln!(f, "maximum bitrate: {}kbps", c / 1024),
            None => Ok(()),
        }
    }
}

/// ### Filetype
impl Tag {
    /// returns the filetype (`ftyp`).
    pub fn filetype(&self) -> &str {
        self.ftyp.as_str()
    }
}
