use crate::constants::{
    GROUND_ALTITUDE_METERS, MAX_ALTITUDE_THRESHOLD, MAX_FREE_FALL_SECONDS,
    SECONDS_TO_CONSIDERED_LANDED,
};
use crate::context::Context;
use crate::data_processor::ProcessorDataPacket;

pub trait State {
    /// Used for updating the struct with new data if we have to.
    fn update_internal(&mut self, processor_data_packet: &ProcessorDataPacket);
    /// Determines if the state should transition to another state based on the context.
    fn should_transition(&self, context: &Context) -> Option<RocketState>;
}

pub enum RocketState {
    Standby(StandbyState),
    Countdown(CountdownState),
    MotorBurn(MotorBurnState),
    CoastState(CoastState),
    FreeFall(FreeFallState),
    Landed(LandedState),
    Shutdown,
}

pub struct StandbyState {}

pub struct CountdownState {}

pub struct MotorBurnState {}

pub struct CoastState {}

pub struct FreeFallState {
    start_time: std::time::Instant,
    landing_timer: Option<std::time::Instant>,
    started_landed_timer: bool,
}

pub struct LandedState {
    start_time: std::time::Instant,
}

impl RocketState {
    pub fn name(&self) -> &'static str {
        match self {
            RocketState::Standby(_) => "Standby",
            RocketState::Countdown(_) => "Countdown",
            RocketState::MotorBurn(_) => "MotorBurn",
            RocketState::CoastState(_) => "Coast",
            RocketState::FreeFall(_) => "FreeFall",
            RocketState::Landed(_) => "Landed",
            RocketState::Shutdown => "Shutdown",
        }
    }

    pub fn update_internal(&mut self, processor_data_packet: &ProcessorDataPacket) {
        match self {
            RocketState::Standby(state) => state.update_internal(processor_data_packet),
            RocketState::Countdown(state) => state.update_internal(processor_data_packet),
            RocketState::MotorBurn(state) => state.update_internal(processor_data_packet),
            RocketState::CoastState(state) => state.update_internal(processor_data_packet),
            RocketState::FreeFall(state) => state.update_internal(processor_data_packet),
            RocketState::Landed(state) => state.update_internal(processor_data_packet),
            RocketState::Shutdown => {}
        }
    }

    pub fn should_transition(&self, context: &Context) -> Option<RocketState> {
        match self {
            RocketState::Standby(state) => state.should_transition(context),
            RocketState::Countdown(state) => state.should_transition(context),
            RocketState::MotorBurn(state) => state.should_transition(context),
            RocketState::CoastState(state) => state.should_transition(context),
            RocketState::FreeFall(state) => state.should_transition(context),
            RocketState::Landed(state) => state.should_transition(context),
            RocketState::Shutdown => None,
        }
    }
}

impl State for StandbyState {
    fn update_internal(&mut self, _: &ProcessorDataPacket) {}
    fn should_transition(&self, _: &Context) -> Option<RocketState> {
        // Unfortunately, the logic for this is in context. I'm not good enough at Rust yet.
        None
    }
}

impl State for CountdownState {
    fn update_internal(&mut self, _: &ProcessorDataPacket) {}
    fn should_transition(&self, context: &Context) -> Option<RocketState> {
        if context.data_processor.current_altitude >= GROUND_ALTITUDE_METERS {
            Some(RocketState::MotorBurn(MotorBurnState {}))
        } else {
            None
        }
    }
}

impl State for MotorBurnState {
    fn update_internal(&mut self, _: &ProcessorDataPacket) {}
    fn should_transition(&self, context: &Context) -> Option<RocketState> {
        if context.data_processor.current_altitude >= context.data_processor.max_altitude * MAX_ALTITUDE_THRESHOLD {
            Some(RocketState::CoastState(CoastState {}))
        } else {
            None
        }
    }
}

impl State for CoastState {
    fn update_internal(&mut self, _: &ProcessorDataPacket) {}
    fn should_transition(&self, context: &Context) -> Option<RocketState> {
        if context.data_processor.current_altitude < context.data_processor.max_altitude * MAX_ALTITUDE_THRESHOLD {
            Some(RocketState::FreeFall(FreeFallState {
                start_time: std::time::Instant::now(),
                landing_timer: None,
                started_landed_timer: false,
            }))
        } else {
            None
        }
    }
}

impl State for FreeFallState {
    fn update_internal(&mut self, processor_data_packet: &ProcessorDataPacket) {
        if processor_data_packet.current_altitude <= GROUND_ALTITUDE_METERS
            && !self.started_landed_timer
        {
            self.landing_timer = Some(std::time::Instant::now());
            self.started_landed_timer = true;
        }
    }

    fn should_transition(&self, _: &Context) -> Option<RocketState> {
        if let Some(timer) = self.landing_timer {
            if timer.elapsed().as_secs() >= SECONDS_TO_CONSIDERED_LANDED {
                return Some(RocketState::Landed(LandedState {
                    start_time: std::time::Instant::now(),
                }));
            }
        } else if self.start_time.elapsed().as_secs() >= MAX_FREE_FALL_SECONDS {
            return Some(RocketState::Landed(LandedState {
                start_time: std::time::Instant::now(),
            }));
        }
        None
    }
}

impl State for LandedState {
    fn update_internal(&mut self, _: &ProcessorDataPacket) {}
    fn should_transition(&self, _: &Context) -> Option<RocketState> {
        // Switch to shutdown state after 5 seconds:
        if self.start_time.elapsed().as_secs() >= 5 {
            return Some(RocketState::Shutdown {});
        }
        None
    }
}
