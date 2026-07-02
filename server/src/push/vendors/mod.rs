//! 各厂商 Push Provider 实现。

pub mod honor;
pub mod huawei;
pub mod meizu;
pub mod oppo;
pub mod vivo;
pub mod xiaomi;

pub use honor::HonorPushProvider;
pub use huawei::HuaweiPushProvider;
pub use meizu::MeizuPushProvider;
pub use oppo::OppoPushProvider;
pub use vivo::VivoPushProvider;
pub use xiaomi::XiaomiPushProvider;
