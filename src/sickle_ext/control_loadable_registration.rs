use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;

use crate::prelude::*;
use crate::sickle_ext::lerp::Lerp;

//-------------------------------------------------------------------------------------------------------------------

/// Helper methods for registering controlled instructions.
pub trait ControlRegistrationExt
{
    fn register_themed<T: GetTypeRegistration + Instruction + ThemedAttribute>(&mut self) -> &mut Self
    where
        <T as ThemedAttribute>::Value: GetTypeRegistration;
    fn register_responsive<T: GetTypeRegistration + Instruction + ThemedAttribute + ResponsiveAttribute>(
        &mut self,
    ) -> &mut Self
    where
        <T as ThemedAttribute>::Value: GetTypeRegistration;
    fn register_animatable<
        T: GetTypeRegistration + Instruction + ThemedAttribute + ResponsiveAttribute + AnimatableAttribute,
    >(
        &mut self,
    ) -> &mut Self
    where
        <T as ThemedAttribute>::Value: Lerp + GetTypeRegistration;
}

impl ControlRegistrationExt for App
{
    fn register_themed<T: GetTypeRegistration + Instruction + ThemedAttribute>(&mut self) -> &mut Self
    where
        <T as ThemedAttribute>::Value: GetTypeRegistration,
    {
        self.register_instruction_type::<T>()
            .register_instruction_type::<Themed<T>>()
            .register_instruction_type::<Multi<Themed<T>>>()
    }

    fn register_responsive<T: GetTypeRegistration + Instruction + ThemedAttribute + ResponsiveAttribute>(
        &mut self,
    ) -> &mut Self
    where
        <T as ThemedAttribute>::Value: GetTypeRegistration,
    {
        self.register_themed::<T>()
            .register_instruction_type::<Responsive<T>>()
            .register_instruction_type::<Multi<Responsive<T>>>()
    }

    fn register_animatable<
        T: GetTypeRegistration + Instruction + ThemedAttribute + ResponsiveAttribute + AnimatableAttribute,
    >(
        &mut self,
    ) -> &mut Self
    where
        <T as ThemedAttribute>::Value: Lerp + GetTypeRegistration,
    {
        self.register_responsive::<T>()
            .register_instruction_type::<Animated<T>>()
            .register_instruction_type::<Multi<Animated<T>>>()
    }
}

//-------------------------------------------------------------------------------------------------------------------
