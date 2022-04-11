use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum EArrayDim {
    NotAnArray = 0,
    TArray = 1,
    CArray = 2,
}

#[derive(Debug, Copy, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ELifetimeCondition {
    // This property has no condition, and will send anytime it changes
    CondNone = 0,
    // This property will only attempt to send on the initial bunch
    CondInitialOnly = 1,
    // This property will only send to the actor's owner
    CondOwnerOnly = 2,
    // This property send to every connection EXCEPT the owner
    CondSkipOwner = 3,
    // This property will only send to simulated actors
    CondSimulatedOnly = 4,
    // This property will only send to autonomous actors
    CondAutonomousOnly = 5,
    // This property will send to simulated OR bRepPhysics actors
    CondSimulatedOrPhysics = 6,
    // This property will send on the initial packet, or to the actors owner
    CondInitialOrOwner = 7,
    // This property has no particular condition, but wants the ability to toggle on/off via SetCustomIsActiveOverride
    CondCustom = 8,
    // This property will only send to the replay connection, or to the actors owner
    CondReplayOrOwner = 9,
    // This property will only send to the replay connection
    CondReplayOnly = 10,
    // This property will send to actors only, but not to replay connections
    CondSimulatedOnlyNoReplay = 11,
    // This property will send to simulated Or bRepPhysics actors, but not to replay connections
    CondSimulatedOrPhysicsNoReplay = 12,
    // This property will not send to the replay connection
    CondSkipReplay = 13,
    // This property will never be replicated
    CondNever = 15,
    CondMax = 16,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EBlueprintTextLiteralType {
    // Text is an empty string. The bytecode contains no strings, and you should use FText::GetEmpty() to initialize the FText instance.
    Empty,
    // Text is localized. The bytecode will contain three strings - source, key, and namespace - and should be loaded via FInternationalization
    LocalizedText,
    // Text is culture invariant. The bytecode will contain one string, and you should use FText::AsCultureInvariant to initialize the FText instance.
    InvariantText,
    // Text is a literal FString. The bytecode will contain one string, and you should use FText::FromString to initialize the FText instance.
    LiteralString,
    // Text is from a string table. The bytecode will contain an object pointer (not used) and two strings - the table ID, and key - and should be found via FText::FromStringTable
    StringTableEntry,
}

#[derive(Debug, Copy, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(i8)]
pub enum TextHistoryType {
    None = -1,
    Base = 0,
    NamedFormat,
    OrderedFormat,
    ArgumentFormat,
    AsNumber,
    AsPercent,
    AsCurrency,
    AsDate,
    AsTime,
    AsDateTime,
    Transform,
    StringTableEntry,
    TextGenerator,
}

impl Default for TextHistoryType {
    fn default() -> Self {
        Self::None
    }
}
