//! Various movies enums

use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Enum MovieScene.EMovieSceneKeyInterpolation
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum EMovieSceneKeyInterpolation {
    /// Auto
    Auto = 0,
    /// User
    User = 1,
    /// Break
    Break = 2,
    /// Linear
    Linear = 3,
    /// Constant
    Constant = 4,
    /// Max
    EMovieSceneKeyInterpolation_MAX = 5,
}

/// Enum MovieScene.EMovieSceneBlendType
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum EMovieSceneBlendType {
    /// Invalid
    Invalid = 0,
    /// Absolute
    Absolute = 1,
    /// Additive
    Additive = 2,
    /// Relative
    Relative = 4,
    /// Max
    EMovieSceneBlendType_MAX = 5,
}

/// Enum MovieScene.EMovieSceneBuiltInEasing
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum EMovieSceneBuiltInEasing {
    /// Linear
    Linear = 0,
    /// Sin in
    SinIn = 1,
    /// Sin out
    SinOut = 2,
    /// Sin in out
    SinInOut = 3,
    /// Quad in
    QuadIn = 4,
    /// Quad out
    QuadOut = 5,
    /// Quad in out
    QuadInOut = 6,
    /// Cubic in
    CubicIn = 7,
    /// Cubic out
    CubicOut = 8,
    /// Cubic in out
    CubicInOut = 9,
    /// Quart in
    QuartIn = 10,
    /// Quart out
    QuartOut = 11,
    /// Quart in out
    QuartInOut = 12,
    /// Quint in
    QuintIn = 13,
    /// Quint out
    QuintOut = 14,
    /// Quint in out
    QuintInOut = 15,
    /// Expo in
    ExpoIn = 16,
    /// Expo out
    ExpoOut = 17,
    /// Expo in out
    ExpoInOut = 18,
    /// Circ in
    CircIn = 19,
    /// Circ out
    CircOut = 20,
    /// Circ in out
    CircInOut = 21,
    /// Max
    EMovieSceneBuiltInEasing_MAX = 22,
}

/// Enum MovieScene.EEvaluationMethod
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum EEvaluationMethod {
    /// Static
    Static = 0,
    /// Swept
    Swept = 1,
    /// Max
    EEvaluationMethod_MAX = 2,
}

/// Enum MovieScene.EUpdateClockSource
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum EUpdateClockSource {
    /// Tick
    Tick = 0,
    /// Platform
    Platform = 1,
    /// Audio
    Audio = 2,
    /// Relative timecode
    RelativeTimecode = 3,
    /// Timecode
    Timecode = 4,
    /// Custom
    Custom = 5,
    /// Max
    EUpdateClockSource_MAX = 6,
}

/// Enum MovieScene.EMovieSceneEvaluationType
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum EMovieSceneEvaluationType {
    /// Frame locked
    FrameLocked = 0,
    /// With sub frames
    WithSubFrames = 1,
    /// Max
    EMovieSceneEvaluationType_MAX = 2,
}

/// Enum MovieScene.EMovieScenePlayerStatus
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum EMovieScenePlayerStatus {
    /// Stopped
    Stopped = 0,
    /// Playing
    Playing = 1,
    /// Recording
    Recording = 2,
    /// Scrubbing
    Scrubbing = 3,
    /// Jumping
    Jumping = 4,
    /// Stepping
    Stepping = 5,
    /// Paused
    Paused = 6,
    /// Max
    MAX = 7,
}

/// Enum MovieScene.EMovieSceneObjectBindingSpace
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum EMovieSceneObjectBindingSpace {
    /// Local
    Local = 0,
    /// Root
    Root = 1,
    /// Max
    EMovieSceneObjectBindingSpace_MAX = 2,
}

/// Enum MovieScene.EMovieSceneCompletionMode
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum EMovieSceneCompletionMode {
    /// Keep state
    KeepState = 0,
    /// Restore state
    RestoreState = 1,
    /// Project default
    ProjectDefault = 2,
    /// Max
    EMovieSceneCompletionMode_MAX = 3,
}

/// Enum MovieScene.ESectionEvaluationFlags
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum ESectionEvaluationFlags {
    /// None
    None = 0,
    /// Pre-roll
    PreRoll = 1,
    /// Post-roll
    PostRoll = 2,
    /// Max
    ESectionEvaluationFlags_MAX = 3,
}

/// Enum MovieScene.EUpdatePositionMethod
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum EUpdatePositionMethod {
    /// Play
    Play = 0,
    /// Jump
    Jump = 1,
    /// Scrub
    Scrub = 2,
    /// Max
    EUpdatePositionMethod_MAX = 3,
}

/// Enum MovieScene.ESpawnOwnership
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum ESpawnOwnership {
    /// Inner sequence
    InnerSequence = 0,
    /// Master sequence
    MasterSequence = 1,
    /// External
    External = 2,
    /// Max
    ESpawnOwnership_MAX = 3,
}
