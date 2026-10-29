//! Profile validation and first-generation state resolution.

use super::{
    ActiveTrinket, Challenge, ChallengeEffect, MapProfile, TrinketEventAction,
    TrinketGenerationEffect, TrinketKind,
};

impl MapProfile {
    pub(crate) fn challenge_mask(&self) -> i32 {
        self.challenges
            .iter()
            .fold(0, |mask, challenge| mask | challenge.mask())
    }

    pub(crate) fn held_at(&self, depth: u32) -> Option<ActiveTrinket> {
        let mut held: Option<ActiveTrinket> = None;
        let mut next_instance = 0;
        for event in self
            .trinket_events
            .iter()
            .take_while(|event| event.before_depth <= depth)
        {
            match event.action {
                TrinketEventAction::Acquired {
                    trinket,
                    min_upgrades,
                } => {
                    next_instance += 1;
                    held = Some(ActiveTrinket {
                        trinket,
                        level: min_upgrades.unwrap_or(0),
                        instance: next_instance,
                    });
                }
                TrinketEventAction::Transmuted { trinket } => {
                    next_instance += 1;
                    held = Some(ActiveTrinket {
                        trinket,
                        level: held.map_or(0, |active| active.level),
                        instance: next_instance,
                    });
                }
                TrinketEventAction::Upgraded => {
                    if let Some(active) = held.as_mut() {
                        active.level += 1;
                    }
                }
            }
        }
        held
    }

    /// Whether any unmodeled player input can affect floor `depth` or an
    /// earlier floor in this chronological profile.
    pub(crate) fn has_unmodeled_generation_inputs_at(&self, depth: u32) -> bool {
        self.challenges
            .iter()
            .any(|challenge| challenge.generation_effect() == ChallengeEffect::Unmodeled)
            || self
                .artifact_events
                .iter()
                .any(|event| event.before_depth <= depth)
            || self
                .trinket_events
                .iter()
                .take_while(|event| event.before_depth <= depth)
                .any(|event| match event.action {
                    TrinketEventAction::Acquired { trinket, .. }
                    | TrinketEventAction::Transmuted { trinket } => {
                        trinket.generation_effect() == TrinketGenerationEffect::Unmodeled
                    }
                    TrinketEventAction::Upgraded => false,
                })
    }

    /// Inputs whose generation effects are intentionally deferred to later
    /// replay phases. Their public projection remains conservative meanwhile.
    pub(crate) fn has_unmodeled_generation_inputs(&self) -> bool {
        self.has_unmodeled_generation_inputs_at(26)
    }

    pub(crate) fn validate(&self, first_effective_depth: u32) -> Result<(), ProfileError> {
        let mut challenge_mask = 0;
        for challenge in &self.challenges {
            let bit = challenge.mask();
            if challenge_mask & bit != 0 {
                return Err(ProfileError::ChallengeRepeated(*challenge));
            }
            challenge_mask |= bit;
        }

        let mut previous_depth = None;
        let mut held = None;
        for event in &self.trinket_events {
            validate_depth(event.before_depth)?;
            if event.before_depth < first_effective_depth {
                return Err(ProfileError::BeforeTrinketAvailable {
                    requested: event.before_depth,
                    earliest: first_effective_depth,
                });
            }
            if previous_depth.is_some_and(|depth| event.before_depth < depth) {
                return Err(ProfileError::TrinketEventsOutOfOrder);
            }
            match event.action {
                TrinketEventAction::Acquired {
                    trinket,
                    min_upgrades,
                } => {
                    if held.is_some() {
                        return Err(ProfileError::TrinketAlreadyHeld);
                    }
                    let level = min_upgrades.unwrap_or(0);
                    if level > 3 {
                        return Err(ProfileError::TrinketLevelOutOfRange);
                    }
                    held = Some((trinket, level));
                }
                TrinketEventAction::Upgraded => match held {
                    None => return Err(ProfileError::TrinketUpgradeWithoutHeld),
                    Some((_, 3)) => return Err(ProfileError::TrinketLevelOutOfRange),
                    Some((trinket, level)) => held = Some((trinket, level + 1)),
                },
                TrinketEventAction::Transmuted { trinket } => match held {
                    None => return Err(ProfileError::TrinketTransmuteWithoutHeld),
                    Some((current, _)) if current == trinket => {
                        return Err(ProfileError::TrinketTransmuteUnchanged)
                    }
                    Some((_, level)) => held = Some((trinket, level)),
                },
            }
            previous_depth = Some(event.before_depth);
        }

        let mut previous_artifact_depth = None;
        for event in &self.artifact_events {
            validate_depth(event.before_depth)?;
            if event.before_depth == 1 {
                return Err(ProfileError::ArtifactBeforeFirstFloor);
            }
            if previous_artifact_depth.is_some_and(|depth| event.before_depth < depth) {
                return Err(ProfileError::ArtifactEventsOutOfOrder);
            }
            previous_artifact_depth = Some(event.before_depth);
        }

        if self
            .claim_state
            .parchment_scrap_level
            .is_some_and(|level| level > 3)
        {
            return Err(ProfileError::ParchmentLevelOutOfRange);
        }
        Ok(())
    }
}

fn validate_depth(depth: u32) -> Result<(), ProfileError> {
    if !(1..=26).contains(&depth) {
        return Err(ProfileError::DepthOutOfRange(depth));
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("challenge {0:?} appears more than once")]
    ChallengeRepeated(Challenge),
    #[error("profile floor {0} is outside the first-generation main-path range")]
    DepthOutOfRange(u32),
    #[error(
        "a trinket event cannot affect floor {requested}; this seed's earliest possible floor is {earliest}"
    )]
    BeforeTrinketAvailable { requested: u32, earliest: u32 },
    #[error("trinket events must be in chronological floor order")]
    TrinketEventsOutOfOrder,
    #[error("an acquisition event requires no trinket to be held")]
    TrinketAlreadyHeld,
    #[error("a trinket upgrade requires a held trinket")]
    TrinketUpgradeWithoutHeld,
    #[error("a trinket cannot be upgraded past +3")]
    TrinketLevelOutOfRange,
    #[error("a trinket transmutation requires a held trinket")]
    TrinketTransmuteWithoutHeld,
    #[error("a trinket transmutation must change to a different trinket")]
    TrinketTransmuteUnchanged,
    #[error("artifact events must be in chronological floor order")]
    ArtifactEventsOutOfOrder,
    #[error("an external artifact event cannot occur before the first floor is generated")]
    ArtifactBeforeFirstFloor,
    #[error("the Trinket Catalyst does not offer {0:?} on this seed")]
    TrinketNotOffered(TrinketKind),
    #[error("Parchment Scrap's claim level must be between +0 and +3")]
    ParchmentLevelOutOfRange,
}
