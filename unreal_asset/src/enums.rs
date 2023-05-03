//! Various UAsset enums

use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Array dimension
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum EArrayDim {
    /// Not an array
    NotAnArray = 0,
    /// Generic array
    TArray = 1,
    /// C Array
    CArray = 2,
}

/// Property lifetime conditions
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ELifetimeCondition {
    /// This property has no condition, and will send anytime it changes
    CondNone = 0,
    /// This property will only attempt to send on the initial bunch
    CondInitialOnly = 1,
    /// This property will only send to the actor's owner
    CondOwnerOnly = 2,
    /// This property send to every connection EXCEPT the owner
    CondSkipOwner = 3,
    /// This property will only send to simulated actors
    CondSimulatedOnly = 4,
    /// This property will only send to autonomous actors
    CondAutonomousOnly = 5,
    /// This property will send to simulated OR bRepPhysics actors
    CondSimulatedOrPhysics = 6,
    /// This property will send on the initial packet, or to the actors owner
    CondInitialOrOwner = 7,
    /// This property has no particular condition, but wants the ability to toggle on/off via SetCustomIsActiveOverride
    CondCustom = 8,
    /// This property will only send to the replay connection, or to the actors owner
    CondReplayOrOwner = 9,
    /// This property will only send to the replay connection
    CondReplayOnly = 10,
    /// This property will send to actors only, but not to replay connections
    CondSimulatedOnlyNoReplay = 11,
    /// This property will send to simulated Or bRepPhysics actors, but not to replay connections
    CondSimulatedOrPhysicsNoReplay = 12,
    /// This property will not send to the replay connection
    CondSkipReplay = 13,
    /// This property will never be replicated
    CondNever = 15,
    /// Max
    CondMax = 16,
}

/// Custom version serialization format
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ECustomVersionSerializationFormat {
    /// Unknown
    Unknown,
    /// Guids
    Guids,
    /// Enums
    Enums,
    /// Optimized
    Optimized,
}
