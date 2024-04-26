#[allow(unused)]
use crate::Arg;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ArgFlags(u32);

impl ArgFlags {
    pub fn set(&mut self, setting: ArgSettings) {
        self.0 |= setting.bit();
    }

    pub fn unset(&mut self, setting: ArgSettings) {
        self.0 &= !setting.bit();
    }

    pub fn is_set(&self, setting: ArgSettings) -> bool {
        self.0 & setting.bit() != 0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

impl std::ops::BitOr for ArgFlags {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self.insert(rhs);
        self
    }
}

/// Various settings that apply to arguments and may be set, unset, and checked via getter/setter
/// methods [`Arg::setting`], [`Arg::unset_setting`], and [`Arg::is_set`]. This is what the
/// [`Arg`] methods which accept a `bool` use internally.
///
/// [`Arg`]: crate::Arg
/// [`Arg::setting`]: crate::Arg::setting()
/// [`Arg::unset_setting`]: crate::Arg::unset_setting()
/// [`Arg::is_set`]: crate::Arg::is_set()
#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum ArgSettings {
    Required,
    Global,
    Hidden,
    NextLineHelp,
    HidePossibleValues,
    AllowHyphenValues,
    AllowNegativeNumbers,
    RequireEquals,
    Last,
    TrailingVarArg,
    HideDefaultValue,
    IgnoreCase,
    #[cfg(feature = "env")]
    HideEnv,
    #[cfg(feature = "env")]
    HideEnvValues,
    HiddenShortHelp,
    HiddenLongHelp,
    Exclusive,
}

impl ArgSettings {
    fn bit(self) -> u32 {
        1 << (self as u8)
    }
}
