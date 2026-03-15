use crate::aop::{AopRenderConfig, FabricProfile};

pub fn printful_crew_neck_default() -> AopRenderConfig {
    let mut config = AopRenderConfig::default();
    config.fabric = FabricProfile::synthetic_aop_default();
    config
}
