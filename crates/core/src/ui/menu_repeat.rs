#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveResult {
    pub index: usize,
    pub changed: bool,
}

pub const DEFAULT_DELAY_MS: u16 = 250;
pub const DELAY_OPTIONS_MS: &[u16] = &[0, 150, 250, 350, 500];

pub struct MenuRepeat;

impl MenuRepeat {
    pub fn sanitize_delay_ms(delay_ms: u16) -> u16 {
        if DELAY_OPTIONS_MS.contains(&delay_ms) {
            delay_ms
        } else {
            DEFAULT_DELAY_MS
        }
    }

    pub fn next_delay_ms(delay_ms: u16) -> u16 {
        let current = Self::sanitize_delay_ms(delay_ms);
        for (i, &opt) in DELAY_OPTIONS_MS.iter().enumerate() {
            if opt == current {
                return DELAY_OPTIONS_MS[(i + 1) % DELAY_OPTIONS_MS.len()];
            }
        }
        DEFAULT_DELAY_MS
    }

    pub fn direction_for_drag(
        delta_x: i32,
        delta_y: i32,
        swipe_threshold_px: u16,
        axis_bias_px: u16,
    ) -> i32 {
        let abs_dx = delta_x.unsigned_abs() as i32;
        let abs_dy = delta_y.unsigned_abs() as i32;
        if abs_dy < swipe_threshold_px as i32 || abs_dy <= abs_dx + axis_bias_px as i32 {
            return 0;
        }
        if delta_y < 0 {
            -1
        } else {
            1
        }
    }

    pub fn is_right_swipe(
        delta_x: i32,
        delta_y: i32,
        swipe_threshold_px: u16,
        axis_bias_px: u16,
    ) -> bool {
        let abs_dx = delta_x.unsigned_abs() as i32;
        let abs_dy = delta_y.unsigned_abs() as i32;
        delta_x >= swipe_threshold_px as i32 && abs_dx > abs_dy + axis_bias_px as i32
    }

    pub fn moved_index(
        selected_index: usize,
        item_count: usize,
        direction: i32,
        wrap: bool,
    ) -> MoveResult {
        let mut result = MoveResult {
            index: selected_index,
            changed: false,
        };
        if direction == 0 || item_count == 0 {
            return result;
        }
        let mut next = selected_index as i64 + direction as i64;
        if wrap {
            if next < 0 {
                next = (item_count as i64) - 1;
            } else if next >= item_count as i64 {
                next = 0;
            }
        } else {
            next = next.clamp(0, (item_count as i64) - 1);
        }
        let next_usize = next as usize;
        result.index = next_usize;
        result.changed = next_usize != selected_index;
        result
    }
}
