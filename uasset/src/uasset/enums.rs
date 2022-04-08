use num_enum::{IntoPrimitive, TryFromPrimitive};


#[derive(Debug, Copy, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum EArrayDim {
    NotAnArray = 0,
    TArray = 1,
    CArray = 2
}

#[derive(Debug, Copy, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ELifetimeCondition
{
    // This property has no condition, and will send anytime it changes
    COND_None = 0,
    // This property will only attempt to send on the initial bunch
    COND_InitialOnly = 1,
    // This property will only send to the actor's owner
    COND_OwnerOnly = 2,
    // This property send to every connection EXCEPT the owner
    COND_SkipOwner = 3,
    // This property will only send to simulated actors
    COND_SimulatedOnly = 4,
    // This property will only send to autonomous actors
    COND_AutonomousOnly = 5,
    // This property will send to simulated OR bRepPhysics actors
    COND_SimulatedOrPhysics = 6,
    // This property will send on the initial packet, or to the actors owner
    COND_InitialOrOwner = 7,
    // This property has no particular condition, but wants the ability to toggle on/off via SetCustomIsActiveOverride
    COND_Custom = 8,
    // This property will only send to the replay connection, or to the actors owner
    COND_ReplayOrOwner = 9,
    // This property will only send to the replay connection
    COND_ReplayOnly = 10,
    // This property will send to actors only, but not to replay connections
    COND_SimulatedOnlyNoReplay = 11,
    // This property will send to simulated Or bRepPhysics actors, but not to replay connections
    COND_SimulatedOrPhysicsNoReplay = 12,
    // This property will not send to the replay connection
    COND_SkipReplay = 13,
    // This property will never be replicated
    COND_Never = 15,
    COND_Max = 16
}


#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
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
    StringTableEntry
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
    TextGenerator
}

impl Default for TextHistoryType {
    fn default() -> Self {
        Self::None
    }
}