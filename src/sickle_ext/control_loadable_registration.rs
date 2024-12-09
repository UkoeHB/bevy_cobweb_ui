use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;

use crate::prelude::*;
use crate::sickle::Lerp;

//-------------------------------------------------------------------------------------------------------------------

/// Helper methods for registering controlled instructions.
pub trait ControlRegistrationExt
{
    fn register_static<T: GetTypeRegistration + Instruction + StaticAttribute>(&mut self) -> &mut Self
    where
        <T as StaticAttribute>::Value: GetTypeRegistration;
    fn register_responsive<T: GetTypeRegistration + Instruction + StaticAttribute + ResponsiveAttribute>(
        &mut self,
    ) -> &mut Self
    where
        <T as StaticAttribute>::Value: GetTypeRegistration;
    fn register_animatable<
        T: GetTypeRegistration + Instruction + StaticAttribute + ResponsiveAttribute + AnimatedAttribute,
    >(
        &mut self,
    ) -> &mut Self
    where
        <T as StaticAttribute>::Value: Lerp + GetTypeRegistration;
}

impl ControlRegistrationExt for App
{
    fn register_static<T: GetTypeRegistration + Instruction + StaticAttribute>(&mut self) -> &mut Self
    where
        <T as StaticAttribute>::Value: GetTypeRegistration,
    {
        self.register_instruction_type::<T>()
            .register_instruction_type::<Static<T>>()
            .register_instruction_type::<Multi<Static<T>>>()
    }

    fn register_responsive<T: GetTypeRegistration + Instruction + StaticAttribute + ResponsiveAttribute>(
        &mut self,
    ) -> &mut Self
    where
        <T as StaticAttribute>::Value: GetTypeRegistration,
    {
        self.register_static::<T>()
            .register_instruction_type::<Responsive<T>>()
            .register_instruction_type::<Multi<Responsive<T>>>()
    }

    fn register_animatable<
        T: GetTypeRegistration + Instruction + StaticAttribute + ResponsiveAttribute + AnimatedAttribute,
    >(
        &mut self,
    ) -> &mut Self
    where
        <T as StaticAttribute>::Value: Lerp + GetTypeRegistration,
    {
        self.register_responsive::<T>()
            .register_instruction_type::<Animated<T>>()
            .register_instruction_type::<Multi<Animated<T>>>()
    }
}

//-------------------------------------------------------------------------------------------------------------------
