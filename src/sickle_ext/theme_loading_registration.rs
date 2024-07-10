use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use sickle_ui::lerp::Lerp;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Helper methods for registering themed loadables.
pub trait ThemedRegistrationExt
{
    fn register_themed<T: GetTypeRegistration + ApplyLoadable + ThemedAttribute>(&mut self) -> &mut Self
    where
        <T as ThemedAttribute>::Value: GetTypeRegistration;
    fn register_responsive<T: GetTypeRegistration + ApplyLoadable + ThemedAttribute + ResponsiveAttribute>(
        &mut self,
    ) -> &mut Self
    where
        <T as ThemedAttribute>::Value: GetTypeRegistration;
    fn register_animatable<
        T: GetTypeRegistration + ApplyLoadable + ThemedAttribute + ResponsiveAttribute + AnimatableAttribute,
    >(
        &mut self,
    ) -> &mut Self
    where
        <T as ThemedAttribute>::Value: Lerp + GetTypeRegistration;
}

impl ThemedRegistrationExt for App
{
    fn register_themed<T: GetTypeRegistration + ApplyLoadable + ThemedAttribute>(&mut self) -> &mut Self
    where
        <T as ThemedAttribute>::Value: GetTypeRegistration,
    {
        self.register_derived::<T>().register_derived::<Themed<T>>()
    }

    fn register_responsive<T: GetTypeRegistration + ApplyLoadable + ThemedAttribute + ResponsiveAttribute>(
        &mut self,
    ) -> &mut Self
    where
        <T as ThemedAttribute>::Value: GetTypeRegistration,
    {
        self.register_themed::<T>()
            .register_derived::<Responsive<T>>()
    }

    fn register_animatable<
        T: GetTypeRegistration + ApplyLoadable + ThemedAttribute + ResponsiveAttribute + AnimatableAttribute,
    >(
        &mut self,
    ) -> &mut Self
    where
        <T as ThemedAttribute>::Value: Lerp + GetTypeRegistration,
    {
        self.register_responsive::<T>()
            .register_derived::<Animated<T>>()
    }
}

//-------------------------------------------------------------------------------------------------------------------
