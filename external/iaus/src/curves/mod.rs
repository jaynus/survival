use std::{
    fmt::Debug,
    ops::{Bound, RangeBounds},
};

pub trait Curve<R>: Debug + Send + Sync
where
    R: RangeBounds<f32> + Debug + Send + Sync,
{
    fn range(&self) -> R;
    fn transform(&self, input: f32) -> f32;

    fn normalize(&self, input: f32) -> f32 {
        let range = self.range();

        let start = match range.start_bound() {
            Bound::Unbounded => std::f32::MIN,
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1.0,
        };

        let end = match range.end_bound() {
            Bound::Unbounded => std::f32::MAX,
            Bound::Included(end) => *end,
            Bound::Excluded(end) => *end - 1.0,
        };

        (input - start) / (end - start)
    }
}

#[derive(Debug)]
pub struct Linear<R>
where
    R: RangeBounds<f32> + Debug + Send + Sync,
{
    pub range: R,
    pub slope: f32,
    pub intercept: f32,
}
impl<R> Curve<R> for Linear<R>
where
    R: RangeBounds<f32> + Debug + Send + Sync + Clone,
{
    #[inline]
    fn range(&self) -> R { self.range.clone() }

    #[inline]
    fn transform(&self, input: f32) -> f32 {
        let input = self.normalize(input);
        (input * self.slope) + self.intercept
    }
}

#[derive(Debug)]
pub struct Exponential<R>
where
    R: RangeBounds<f32> + Debug + Send + Sync,
{
    pub range: R,
    pub power: f32,
}
impl<R> Curve<R> for Exponential<R>
where
    R: RangeBounds<f32> + Debug + Send + Sync + Clone,
{
    #[inline]
    fn range(&self) -> R { self.range.clone() }

    #[inline]
    fn transform(&self, input: f32) -> f32 {
        let input = self.normalize(input);
        input.powf(self.power)
    }
}

#[derive(Debug)]
pub struct ExponentialDecay<R>
where
    R: RangeBounds<f32> + Debug + Send + Sync,
{
    pub range: R,
    pub magnitude: f32,
}
impl<R> Curve<R> for ExponentialDecay<R>
where
    R: RangeBounds<f32> + Debug + Send + Sync + Clone,
{
    #[inline]
    fn range(&self) -> R { self.range.clone() }

    #[inline]
    fn transform(&self, input: f32) -> f32 {
        let input = self.normalize(input);
        self.magnitude.powf(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curves;
    use plotters::prelude::*;

    #[test]
    fn plot_sigmoid() -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new("sigmoid.png", (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption("1 / 1 + e^x", ("Arial", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_ranged(0f32..1f32, 0f32..1f32)?;

        chart.configure_mesh().draw()?;

        let curve = curves::Exponential {
            range: std::ops::Range {
                start: 0.0,
                end: 256.0,
            },
            power: 2.0,
        };

        chart
            .draw_series(LineSeries::new(
                (0..=255)
                    .map(|x| x as f32)
                    .map(|x| (x / 255.0, curve.transform(x))),
                &RED,
            ))?
            .label("Sigmoid Curve")
            .legend(|(x, y)| Path::new(vec![(x, y), (x, y)], &RED));

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        Ok(())
    }
}
