use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeTools {
    pub cargo: std::path::PathBuf,
    pub path: std::ffi::OsString,
    pub library_path: std::ffi::OsString,
    pub kernel_identity: String,
    pub cpu_identity: String,
    pub governor: String,
    pub load_class: String,
}

pub fn production_tools() -> Result<RuntimeTools, String> {
    Ok(RuntimeTools {
        cargo: required_absolute_path("OCTET_BENCHMARK_CARGO")?,
        path: required_environment("OCTET_BENCHMARK_PATH")?,
        library_path: required_environment("OCTET_BENCHMARK_LD_LIBRARY_PATH")?,
        kernel_identity: required_utf8_environment("OCTET_BENCHMARK_KERNEL_IDENTITY")?,
        cpu_identity: required_utf8_environment("OCTET_BENCHMARK_CPU_IDENTITY")?,
        governor: required_utf8_environment("OCTET_BENCHMARK_GOVERNOR")?,
        load_class: required_utf8_environment("OCTET_BENCHMARK_LOAD_CLASS")?,
    })
}

pub(super) fn environment_observation(
    profile: &Profile,
    workload: &WorkloadProfile,
    tools: &RuntimeTools,
) -> EnvironmentObservation {
    EnvironmentObservation {
        environment_identity: workload.environment_identity.clone(),
        target: format!("{}-unknown-{}", std::env::consts::ARCH, std::env::consts::OS),
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        kernel_identity: tools.kernel_identity.clone(),
        cpu_identity: tools.cpu_identity.clone(),
        governor: tools.governor.clone(),
        load_class: tools.load_class.clone(),
        rust_toolchain: profile.rust_toolchain.clone(),
        nix_closure_identity: profile.provider.closure_identity.clone(),
        isolation: "bounded-exec-process-group-clear-environment".to_owned(),
        cpu_time_supported: false,
        peak_memory_supported: false,
    }
}

fn required_absolute_path(name: &str) -> Result<std::path::PathBuf, String> {
    let path =
        std::path::PathBuf::from(std::env::var_os(name).ok_or_else(|| format!("benchmark runtime is missing {name}"))?);
    if !path.is_absolute() {
        return Err(format!("benchmark runtime {name} is not absolute"));
    }
    Ok(path)
}

fn required_environment(name: &str) -> Result<std::ffi::OsString, String> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("benchmark runtime is missing {name}"))
}

fn required_utf8_environment(name: &str) -> Result<String, String> {
    required_environment(name)?
        .into_string()
        .map_err(|_| format!("benchmark runtime {name} is not valid UTF-8"))
}
