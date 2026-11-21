use crate::Number;

#[derive(Clone, Copy)]
pub(crate) struct RpnStack {
    values: [Number; 4],
    last_x: Number,
}

impl RpnStack {
    pub(crate) fn new(zero: Number) -> Self {
        Self {
            values: [zero; 4],
            last_x: zero,
        }
    }

    pub(crate) fn values(&self) -> [Number; 4] {
        self.values
    }

    pub(crate) fn x(&self) -> Number {
        self.values[0]
    }

    pub(crate) fn y(&self) -> Number {
        self.values[1]
    }

    pub(crate) fn last_x(&self) -> Number {
        self.last_x
    }

    pub(crate) fn lift(&mut self) {
        let [x, y, z, _] = self.values;
        self.values = [x, x, y, z];
    }

    pub(crate) fn drop_stack(&mut self) {
        let [_, y, z, t] = self.values;
        self.values = [y, z, t, t];
    }

    pub(crate) fn swap_xy(&mut self) {
        self.values.swap(0, 1);
    }

    pub(crate) fn roll_up(&mut self) {
        self.values.rotate_right(1);
    }

    pub(crate) fn roll_down(&mut self) {
        self.values.rotate_left(1);
    }

    pub(crate) fn clear_values(&mut self, zero: Number) {
        self.values.fill(zero);
    }

    pub(crate) fn set_x(&mut self, value: Number) {
        self.values[0] = value;
    }

    pub(crate) fn set_last_x(&mut self, value: Number) {
        self.last_x = value;
    }

    pub(crate) fn try_map(
        &self,
        mut convert: impl FnMut(Number) -> Result<Number, crate::CalcError>,
    ) -> Result<Self, crate::CalcError> {
        let [x, y, z, t] = self.values;
        Ok(Self {
            values: [convert(x)?, convert(y)?, convert(z)?, convert(t)?],
            last_x: convert(self.last_x)?,
        })
    }
}
