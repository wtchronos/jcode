/// Live telemetry shown by Desktop2's compact run strip and expanded panel.
///
/// Providers may send several cumulative usage snapshots during one API call.
/// The `last_*` fields turn those snapshots into deltas so the session totals
/// never double-count a late/final report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hud {
    expanded: bool,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_reported_input_tokens: u64,
    cache_reported: bool,
    latest_proof: Option<String>,
    last_input: Option<u64>,
    last_output: Option<u64>,
    last_cache_read: Option<u64>,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            expanded: true,
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_reported_input_tokens: 0,
            cache_reported: false,
            latest_proof: None,
            last_input: None,
            last_output: None,
            last_cache_read: None,
        }
    }
}

impl Hud {
    pub fn record_usage(&mut self, input: u64, output: u64, cache_read_input: Option<u64>) {
        let input_delta = snapshot_delta(self.last_input, input);
        let output_delta = snapshot_delta(self.last_output, output);
        self.input_tokens = self.input_tokens.saturating_add(input_delta);
        self.output_tokens = self.output_tokens.saturating_add(output_delta);
        self.last_input = Some(input);
        self.last_output = Some(output);

        if let Some(cache_read) = cache_read_input {
            self.cache_reported = true;
            self.cache_reported_input_tokens =
                self.cache_reported_input_tokens.saturating_add(input_delta);
            self.cache_read_tokens = self
                .cache_read_tokens
                .saturating_add(snapshot_delta(self.last_cache_read, cache_read));
            self.last_cache_read = Some(cache_read);
        }
    }

    /// Start the next turn with fresh provider-snapshot watermarks while
    /// preserving the session totals shown in the instrument panel.
    pub fn finish_turn(&mut self) {
        self.last_input = None;
        self.last_output = None;
        self.last_cache_read = None;
    }

    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }

    pub fn expanded(&self) -> bool {
        self.expanded
    }

    pub fn input_tokens(&self) -> u64 {
        self.input_tokens
    }

    pub fn output_tokens(&self) -> u64 {
        self.output_tokens
    }

    pub fn cache_read_tokens(&self) -> Option<u64> {
        self.cache_reported.then_some(self.cache_read_tokens)
    }

    pub fn cache_hit_pct(&self) -> Option<u8> {
        if !self.cache_reported || self.cache_reported_input_tokens == 0 {
            return None;
        }
        let numerator = self.cache_read_tokens.saturating_mul(100);
        let ratio = numerator.saturating_add(self.cache_reported_input_tokens / 2)
            / self.cache_reported_input_tokens;
        Some(ratio.min(100) as u8)
    }

    pub fn has_usage(&self) -> bool {
        self.input_tokens > 0 || self.output_tokens > 0
    }

    pub fn note_completion(&mut self, label: &str, summary: &str) {
        self.latest_proof = Some(format!("{} · {}", label.trim(), summary.trim()));
    }

    pub fn latest_proof(&self) -> Option<&str> {
        self.latest_proof.as_deref()
    }
}

fn snapshot_delta(previous: Option<u64>, current: u64) -> u64 {
    previous
        .filter(|previous| current >= *previous)
        .map_or(current, |previous| current - previous)
}

pub fn compact_tokens(tokens: u64) -> String {
    if tokens < 1_000 {
        return tokens.to_string();
    }
    let (value, suffix) = if tokens < 1_000_000 {
        (tokens as f64 / 1_000.0, "k")
    } else {
        (tokens as f64 / 1_000_000.0, "m")
    };
    format!("{value:.1}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::Hud;

    #[test]
    fn missing_cache_telemetry_stays_unknown() {
        let mut hud = Hud::default();
        hud.record_usage(1_000, 100, None);

        assert_eq!(hud.input_tokens(), 1_000);
        assert_eq!(hud.output_tokens(), 100);
        assert_eq!(hud.cache_hit_pct(), None);
    }

    #[test]
    fn cache_telemetry_accumulates_and_is_bounded() {
        let mut hud = Hud::default();
        hud.record_usage(1_000, 100, Some(900));
        hud.record_usage(500, 50, Some(400));

        assert_eq!(hud.input_tokens(), 1_500);
        assert_eq!(hud.output_tokens(), 150);
        assert_eq!(hud.cache_read_tokens(), Some(1_300));
        assert_eq!(hud.cache_hit_pct(), Some(87));

        hud.record_usage(100, 10, Some(10_000));
        assert_eq!(hud.cache_hit_pct(), Some(100));
    }

    #[test]
    fn toggling_the_panel_preserves_telemetry() {
        let mut hud = Hud::default();
        hud.record_usage(42_800, 3_100, Some(38_948));
        assert!(hud.expanded());

        hud.toggle();
        assert!(!hud.expanded());
        assert_eq!(hud.input_tokens(), 42_800);
        assert_eq!(hud.output_tokens(), 3_100);
        assert_eq!(hud.cache_hit_pct(), Some(91));
    }

    #[test]
    fn compact_token_counts_keep_the_instrument_panel_short() {
        assert_eq!(super::compact_tokens(999), "999");
        assert_eq!(super::compact_tokens(1_500), "1.5k");
        assert_eq!(super::compact_tokens(42_800), "42.8k");
        assert_eq!(super::compact_tokens(1_250_000), "1.2m");
    }

    #[test]
    fn completed_background_work_becomes_the_latest_proof() {
        let mut hud = Hud::default();
        hud.note_completion("cargo test", "✓ completed · 27 tests passed");

        assert_eq!(
            hud.latest_proof(),
            Some("cargo test · ✓ completed · 27 tests passed")
        );
    }
}
