use bevy::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Value for use in 2D UI.
///
/// Defaults to `Self::Px(0.)`.
#[derive(Reflect, Default, Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Val2d
{
    /// Absolute pixels in [`Transform`] coordinates.
    #[default]
    Px(f32),
    /// A percent of another value in [`Transform`] coordinates.
    Percent(f32),
    /// An infinite value.
    Inf,
    /// A negative infinite value.
    NegInf,
}

impl Val2d
{
    /// Converts to a value in [`Transform`] coordinates.
    ///
    /// Will return `0.` instead of `NaN` if necessary.
    pub fn compute(&self, ref_px: f32) -> f32
    {
        match self
        {
            Self::Px(px) => fix_nan(px),
            Self::Percent(percent) => fix_nan(ref_px)*fix_nan(percent),
            Self::Inf => f32::INFINITY,
            Self::NegInf => f32::NEG_INFINITY,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns `0.` if `v` is `NaN`. Otherwise returns `v`.
pub fn fix_nan(v: f32) -> f32
{
    if v.is_nan() {
        return 0.
    }
    v
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct Val2dPlugin;

impl Plugin for Val2dPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Val2d>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
