use super::*;
impl RunnerSettings {
    pub fn new(transmission_delay: Duration, n_ack: usize) -> Self {
        RunnerSettings {
            transmission_delay,
            n_ack,
        }
    }

    pub fn default() -> Self {
        RunnerSettings::new(DEFAULT_TRANSMISSION_DELAY, 0)
    }

    pub fn from_system(system_settings: &SystemSettings) -> Self {
        RunnerSettings::new(system_settings.transmition_delay, system_settings.n_ack)
    }
}
