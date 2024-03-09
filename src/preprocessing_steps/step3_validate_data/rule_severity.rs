pub trait Severity {}

pub struct FatalSeverity {}
impl Severity for FatalSeverity {}

pub struct HighSeverity {}
impl Severity for HighSeverity {}

pub struct LowSeverity {}
impl Severity for LowSeverity {}