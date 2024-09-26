use crate::css::{
    Animation, AnimationTrack, ComputedStyle, Declaration, PropertyDescriptor, PropertyKey,
};
use crate::styles::initial::initial;
use crate::styles::Cascade;
use log::error;
use std::collections::{BTreeMap, HashMap, HashSet};

impl<'c> Cascade<'c> {
    pub(crate) fn compute_animation_tracks(
        &self,
        animation: &Animation,
        style: &ComputedStyle,
    ) -> Vec<AnimationTrack> {
        let mut animated_properties: HashSet<PropertyDescriptor> = HashSet::new();
        let mut computed_keyframe_styles = BTreeMap::new();
        for keyframe in &animation.keyframes {
            let mut keyframe_style = HashMap::new();
            for declaration in &keyframe.declaration {
                match declaration {
                    Declaration::Variable(variable) => {
                        error!(
                            "can't define variable {} in animation {} keyframe {}, not supported",
                            variable.key, animation.name, keyframe.step
                        )
                    }
                    Declaration::Property(property) => {
                        for index in 0..property.values.len() {
                            let value = &property.values[index];
                            self.compute_style(property.key, index, value, &mut keyframe_style);
                        }
                    }
                }
            }
            animated_properties.extend(keyframe_style.keys());
            computed_keyframe_styles.insert(keyframe.step, keyframe_style);
        }
        let from = computed_keyframe_styles.entry(0).or_default();
        for property in &animated_properties {
            if !from.contains_key(property) {
                let value = match style.get(property) {
                    Some(value) => value.clone(),
                    None => initial(property.key),
                };
                from.insert(*property, value);
            }
        }
        let to = computed_keyframe_styles.entry(100).or_default();
        for property in &animated_properties {
            if !to.contains_key(&property) {
                let value = match style.get(&property) {
                    Some(value) => value.clone(),
                    None => initial(property.key),
                };
                to.insert(*property, value);
            }
        }
        let mut tracks: HashMap<PropertyDescriptor, AnimationTrack> = HashMap::new();
        for (step, style) in computed_keyframe_styles {
            for (property, value) in style {
                let track =
                    tracks
                        .entry(property)
                        .or_insert_with_key(|descriptor| AnimationTrack {
                            descriptor: *descriptor,
                            frames: BTreeMap::new(),
                        });
                track.frames.insert(step, value);
            }
        }
        tracks.into_values().collect()
    }
}
