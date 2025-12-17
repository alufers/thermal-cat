use std::time::{Duration, Instant};

use crate::temperature::{Temp, TempRange};

//
// Struct holding the state of the auto temp range algorithm.
//
pub struct AutoDisplayRangeController {
    current: TempRange,

    last_compute_call: Option<Instant>,

    anim_target_range: Option<TempRange>,
    anim_progress: Duration,

    clipping_time: Duration,

    // Settings
    clipping_time_threshold: Duration,
    anim_duration: Duration,

    // headroom to add to captured range when setting new range
    new_range_max_headroom: Temp,
    new_range_min_headroom: Temp,

    shrink_range_max_headroom: Temp,
    shrink_range_min_headroom: Temp,

    min_separation: Temp,
}

impl AutoDisplayRangeController {
    pub fn new() -> AutoDisplayRangeController {
        AutoDisplayRangeController {
            current: TempRange::new(Temp::from_celsius(0.0), Temp::from_celsius(50.0)),
            last_compute_call: None,
            anim_target_range: None,
            anim_progress: Duration::from_secs(0),
            clipping_time: Duration::from_secs(0),

            clipping_time_threshold: Duration::from_millis(900),
            anim_duration: Duration::from_millis(500),

            new_range_max_headroom: Temp::new(1.0),
            new_range_min_headroom: Temp::new(1.0),

            shrink_range_max_headroom: Temp::new(2.0),
            shrink_range_min_headroom: Temp::new(2.0),
            min_separation: Temp::new(5.0),
        }
    }

    pub fn compute(&mut self, captured_range: TempRange) -> TempRange {
        let now = Instant::now();
        let last_compute_call = self.last_compute_call.unwrap_or(now);
        let delta = now - last_compute_call;
        self.last_compute_call = Some(now);

        // if the max point or min point of the captured range is in the shrinking range, start shrinking the current range
        let shrinking_range = TempRange::new(
            self.current.min + self.shrink_range_min_headroom,
            self.current.max - self.shrink_range_max_headroom,
        );

        // check clipping
        if !self.current.contains_range(captured_range)
            || captured_range.max < shrinking_range.max
            || captured_range.min > shrinking_range.min
        {
            self.clipping_time += delta;
            // if we are in the shrinking range, reset the clipping time
            self.clipping_time += delta;
        } else {
            self.clipping_time = Duration::from_secs(0);
        }

        if self.clipping_time > self.clipping_time_threshold {
            if self.anim_target_range.is_none() {
                let target = TempRange::new(
                    captured_range.min - self.new_range_min_headroom,
                    captured_range.max + self.new_range_max_headroom,
                );
                self.anim_target_range = Some(target);

                self.anim_progress = Duration::from_secs(0);
            }
        } else {
            self.anim_target_range = None;
        }

        if let Some(target_range) = self.anim_target_range {
            self.anim_progress += delta;
            let factor = self.anim_progress.as_secs_f32() / self.anim_duration.as_secs_f32();
            self.current = self.current.animate(target_range, factor);
            if factor >= 1.0 {
                self.anim_target_range = None;
            }
        }

        // at the end apply min separation
        if self.current.diff() < self.min_separation {
            TempRange::new(
                self.current.min,
                self.current.max + (self.min_separation - self.current.diff()),
            )
        } else {
            self.current
        }
    }
}
