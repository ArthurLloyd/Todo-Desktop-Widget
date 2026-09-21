// Development builds already inherit the terminal. Release builds deliberately
// keep the Windows GUI subsystem console-free; the legacy setting remains only
// for backward-compatible settings deserialization.
pub fn apply(_enabled: bool) {}
