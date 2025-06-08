# M2 Anim Format 🎬

M2 .anim files contain external animation sequences that can be loaded on demand
for M2 models.

## Overview

- **Extension**: `.anim`
- **Purpose**: Store animations separately from main M2 file
- **Introduced**: Wrath of the Lich King (3.0.2)
- **Benefits**: Reduced memory usage, faster loading, animation sharing
- **Naming**: `<ModelName><AnimID>-<SubAnimID>.anim`

## Structure

### File Naming Convention

```text
// Format: ModelName + AnimID + SubAnimID
Character/BloodElf/Female/BloodElfFemale0060-00.anim
                                         ^^^^ ^^
                                         |    |
                                         |    SubAnimID (variation)
                                         AnimID (animation type)
```

### Anim File Structure

```rust
struct AnimFileHeader {
    version: u32,           // Always 0x100 (version 1.0)
    sub_version: u32,       // Always 0
}

struct AnimFileData {
    global_sequences: Vec<GlobalSequence>,
    animations: Vec<M2Animation>,
    animation_lookups: Vec<i16>,
    play_anim_combos: Vec<PlayAnimCombo>,
    rel_anim_combos: Vec<RelAnimCombo>,
    bones: Vec<M2Bone>,
    key_bone_lookups: Vec<i16>,
    vertices: Vec<M2Vertex>,
    colors: Vec<M2Color>,
    textures: Vec<M2Texture>,
    texture_weights: Vec<M2TextureWeight>,
    texture_transforms: Vec<M2TextureTransform>,
    texture_combos: Vec<u16>,
    materials: Vec<M2Material>,
    material_combos: Vec<u16>,
    texture_coord_combos: Vec<u16>,
    fake_anim_ids: Vec<u16>,
    attachments: Vec<M2Attachment>,
}
```

## Usage Example

```rust
use warcraft_rs::m2::{M2Model, AnimLoader, AnimationId};

// Load base model
let mut model = M2Model::open("Character/Human/Male/HumanMale.m2")?;

// Load external animation
let anim_loader = AnimLoader::new();
let dance_anim = anim_loader.load_animation(&model, AnimationId::Dance)?;

// Apply animation to model
model.add_external_animation(dance_anim);

// Play the loaded animation
model.play_animation(AnimationId::Dance);

// Batch load animations
let combat_anims = anim_loader.load_animation_set(&model, &[
    AnimationId::Attack1H,
    AnimationId::Attack2H,
    AnimationId::AttackOff,
    AnimationId::Parry,
])?;

for anim in combat_anims {
    model.add_external_animation(anim);
}
```

## Animation ID Mapping

### Common Animation IDs

```rust
// Animation IDs that often have .anim files
const EMOTE_ANIMS: &[u16] = &[
    60,   // UseStanding
    61,   // Exclamation
    62,   // Question
    63,   // Bow
    64,   // Wave
    65,   // Cheer
    66,   // Dance
    67,   // Laugh
    68,   // Sleep
    69,   // SitGround
    // ... more emotes
];

const COMBAT_ANIMS: &[u16] = &[
    160,  // Attack1H
    161,  // Attack2H
    162,  // Attack2HL
    163,  // AttackUnarmed
    164,  // AttackBow
    165,  // AttackRifle
    166,  // AttackThrown
    // ... more combat
];
```

## Advanced Features

### Animation Sharing

```rust
// Share animations between similar models
struct AnimationCache {
    cache: HashMap<String, Arc<AnimationData>>,
}

impl AnimationCache {
    fn get_shared_animation(&self, model_type: &str, anim_id: u16) -> Option<Arc<AnimationData>> {
        // Try exact match first
        let exact_key = format!("{}_{}", model_type, anim_id);
        if let Some(anim) = self.cache.get(&exact_key) {
            return Some(anim.clone());
        }

        // Try generic version (e.g., all humans share some animations)
        let race = extract_race(model_type);
        let generic_key = format!("{}_{}", race, anim_id);
        self.cache.get(&generic_key).cloned()
    }
}
```

### Lazy Loading

```rust
struct LazyAnimationLoader {
    model: M2Model,
    loaded_anims: HashMap<u16, AnimationData>,
    anim_paths: HashMap<u16, String>,
}

impl LazyAnimationLoader {
    fn play_animation(&mut self, anim_id: u16) -> Result<()> {
        // Load animation on first use
        if !self.loaded_anims.contains_key(&anim_id) {
            if let Some(path) = self.anim_paths.get(&anim_id) {
                let anim = load_anim_file(path)?;
                self.loaded_anims.insert(anim_id, anim);
            }
        }

        // Use loaded animation
        if let Some(anim) = self.loaded_anims.get(&anim_id) {
            self.model.apply_animation(anim);
        }

        Ok(())
    }
}
```

### Animation Streaming

```rust
// Stream animations for large cutscenes
struct AnimationStreamer {
    current_segment: Option<AnimSegment>,
    next_segment: Option<AnimSegment>,
    preload_time: u32,  // Milliseconds before needed
}

impl AnimationStreamer {
    fn update(&mut self, current_time: u32) -> Result<()> {
        // Check if we need to preload next segment
        if let Some(current) = &self.current_segment {
            let time_until_end = current.end_time - current_time;

            if time_until_end <= self.preload_time && self.next_segment.is_none() {
                // Start async load of next segment
                self.start_preload_next_segment()?;
            }
        }

        // Switch to next segment if current is done
        if current_time >= self.current_segment.as_ref().unwrap().end_time {
            self.current_segment = self.next_segment.take();
        }

        Ok(())
    }
}
```

## Common Patterns

### Animation Discovery

```rust
fn discover_animations(model_path: &str) -> Vec<AnimationFile> {
    let model_name = Path::new(model_path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    let model_dir = Path::new(model_path).parent().unwrap();
    let mut animations = Vec::new();

    // Search for .anim files matching pattern
    for entry in fs::read_dir(model_dir)? {
        let path = entry?.path();
        if path.extension() == Some(OsStr::new("anim")) {
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                if file_name.starts_with(model_name) {
                    // Extract animation ID from filename
                    if let Some(anim_id) = parse_anim_id(&file_name) {
                        animations.push(AnimationFile {
                            path,
                            anim_id,
                        });
                    }
                }
            }
        }
    }

    animations
}
```

### Memory Management

```rust
struct AnimationManager {
    memory_limit: usize,
    loaded_anims: LruCache<(String, u16), AnimationData>,
}

impl AnimationManager {
    fn load_animation(&mut self, model: &str, anim_id: u16) -> Result<&AnimationData> {
        let key = (model.to_string(), anim_id);

        if !self.loaded_anims.contains(&key) {
            let anim = load_anim_file(&format!("{}{:04}-00.anim", model, anim_id))?;

            // Check memory usage
            while self.get_memory_usage() + anim.size() > self.memory_limit {
                // Evict least recently used
                self.loaded_anims.pop_lru();
            }

            self.loaded_anims.put(key.clone(), anim);
        }

        Ok(self.loaded_anims.get(&key).unwrap())
    }
}
```

## Performance Tips

- Load animations asynchronously during loading screens
- Cache frequently used animations (idle, walk, run)
- Unload rarely used animations (emotes, special attacks)
- Consider animation LOD for distant models

## Common Issues

### Missing Animations

- Not all animations are externalized
- Some remain embedded in M2 file
- Check both internal and external sources

### Version Compatibility

- .anim format introduced in WotLK (3.0.2)
- Earlier clients use embedded animations only
- Format unchanged through 5.4.8

### File Discovery

- Animation files follow strict naming
- Sub-animations use different SubAnimID
- Some animations have multiple variations

## References

- [M2/.anim Format (wowdev.wiki)](https://wowdev.wiki/M2/.anim)
- [Animation IDs](https://wowdev.wiki/M2#Animation_sequences)

## See Also

- [M2 Format](m2.md) - Main model format
- [Animation System Guide](../../guides/animation-system.md)
- [Memory Management Guide](../../guides/memory-management.md)
