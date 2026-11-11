#![feature(register_tool)]
#![register_tool(tigerstyle)]

pub enum DeploymentMode {
    Safe,
    Fast,
}

pub struct DeploymentOptions {
    pub mode: DeploymentMode,
    pub audit: bool,
}

pub struct BoolFlag(pub bool);

pub fn deploy(options: DeploymentOptions) -> bool {
    matches!(options.mode, DeploymentMode::Safe) && options.audit
}

pub fn wrapped(flag: BoolFlag, optional: Option<bool>) {
    let _ = (flag, optional);
}

fn private_predicate(enabled: bool) -> bool {
    enabled
}

mod hidden {
    pub fn callable(enabled: bool) {
        let _ = enabled;
    }

    pub struct Controller;

    impl Controller {
        pub fn set(&self, enabled: bool) {
            let _ = enabled;
        }
    }
}

struct PrivateController;

impl PrivateController {
    pub fn set(&self, enabled: bool) {
        let _ = enabled;
    }
}

pub struct ExternalMode(bool);

impl From<bool> for ExternalMode {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

macro_rules! generated {
    () => {
        pub fn generated_api(enabled: bool) {
            let _ = enabled;
        }
    };
}
generated!();

fn main() {
    let options = DeploymentOptions {
        mode: DeploymentMode::Safe,
        audit: private_predicate(true),
    };
    assert!(deploy(options));
    wrapped(BoolFlag(false), Some(true));
    assert!(ExternalMode::from(false).0 == false);
    generated_api(true);
    hidden::callable(true);
    hidden::Controller.set(true);
    PrivateController.set(true);
}
